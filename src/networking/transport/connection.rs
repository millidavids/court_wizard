//! iroh connection management — endpoint creation, address exchange, and I/O loops.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use crossbeam_channel::{Receiver, Sender};
use iroh::endpoint::Connection;
use iroh::{Endpoint, EndpointAddr, endpoint::presets};
use tokio::sync::Notify;
use tracing::warn;

use crate::networking::resources::ConnectionState;

use super::codec::{self, DatagramReassembler};
use super::runtime::{TransportCommand, TransportEvent};

/// Application-Layer Protocol Negotiation identifier for Court Wizard P2P.
const ALPN: &[u8] = b"court-wizard/1";

/// Default max datagram payload size if the connection doesn't report one.
const DEFAULT_MAX_DATAGRAM: usize = 1200;

/// Main entry point for the transport runtime. Runs on the tokio thread and
/// processes commands from the Bevy bridge.
pub(super) async fn run_transport(
    mut command_rx: tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
    event_tx: Sender<TransportEvent>,
    reliable_rx: Receiver<Vec<u8>>,
    unreliable_rx: Receiver<Vec<u8>>,
    reliable_notify: Arc<Notify>,
    unreliable_notify: Arc<Notify>,
) {
    loop {
        let cmd = match command_rx.recv().await {
            Some(cmd) => cmd,
            None => return, // Channel closed — Bevy is shutting down.
        };

        match cmd {
            TransportCommand::CreateHost { use_relay } => {
                handle_host(
                    use_relay,
                    &mut command_rx,
                    &event_tx,
                    &reliable_rx,
                    &unreliable_rx,
                    &reliable_notify,
                    &unreliable_notify,
                )
                .await;
            }
            TransportCommand::ConnectToHost { ticket_code } => {
                handle_guest(
                    &ticket_code,
                    &mut command_rx,
                    &event_tx,
                    &reliable_rx,
                    &unreliable_rx,
                    &reliable_notify,
                    &unreliable_notify,
                )
                .await;
            }
            TransportCommand::Disconnect => {
                // Nothing to disconnect — we're idle.
            }
        }
    }
}

/// Host flow: create endpoint, generate connection code, wait for guest, run I/O.
async fn handle_host(
    use_relay: bool,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
    event_tx: &Sender<TransportEvent>,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) {
    let ep = if use_relay {
        Endpoint::builder(presets::N0)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
    } else {
        Endpoint::builder(presets::N0DisableRelay)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
    };

    let ep = match ep {
        Ok(ep) => ep,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Failed to create endpoint: {e}"));
            return;
        }
    };

    // Wait for relay connectivity (online mode) with a timeout.
    if use_relay
        && tokio::time::timeout(std::time::Duration::from_secs(10), ep.online())
            .await
            .is_err()
    {
        warn!("Timed out waiting for relay, proceeding with local addresses only");
    }

    let addr = ep.addr();
    let ticket_str = encode_endpoint_addr(&addr);
    send_event(event_tx, TransportEvent::LocalCode(ticket_str));

    // Wait for guest to connect, or a Disconnect command.
    let accept_handle = tokio::spawn({
        let ep = ep.clone();
        async move { ep.accept().await }
    });

    let conn = tokio::select! {
        result = accept_handle => {
            match result {
                Ok(Some(incoming)) => match incoming.await {
                    Ok(conn) => conn,
                    Err(e) => {
                        send_error_and_fail(event_tx, format!("Guest connection failed: {e}"));
                        let _ = ep.close().await;
                        return;
                    }
                },
                Ok(None) => {
                    send_error_and_fail(event_tx, "Endpoint closed before guest connected".into());
                    return;
                }
                Err(e) => {
                    send_error_and_fail(event_tx, format!("Accept task panicked: {e}"));
                    return;
                }
            }
        }
        _ = wait_for_disconnect(command_rx) => {
            let _ = ep.close().await;
            send_event(event_tx, TransportEvent::StateChanged(ConnectionState::Disconnected));
            return;
        }
    };

    send_event(
        event_tx,
        TransportEvent::StateChanged(ConnectionState::Connected),
    );

    // Host opens the bidirectional stream.
    run_connection_io(
        conn,
        &ep,
        true,
        command_rx,
        event_tx,
        reliable_rx,
        unreliable_rx,
        reliable_notify,
        unreliable_notify,
    )
    .await;

    let _ = ep.close().await;
}

/// Guest flow: parse connection code, connect to host, run I/O.
async fn handle_guest(
    ticket_code: &str,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
    event_tx: &Sender<TransportEvent>,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) {
    send_event(
        event_tx,
        TransportEvent::StateChanged(ConnectionState::Connecting),
    );

    let endpoint_addr = match decode_endpoint_addr(ticket_code.trim()) {
        Ok(addr) => addr,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Invalid connection code: {e}"));
            return;
        }
    };

    let ep = match Endpoint::builder(presets::N0).bind().await {
        Ok(ep) => ep,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Failed to create endpoint: {e}"));
            return;
        }
    };

    let conn = match ep.connect(endpoint_addr, ALPN).await {
        Ok(conn) => conn,
        Err(e) => {
            send_error_and_fail(event_tx, format!("Failed to connect to host: {e}"));
            let _ = ep.close().await;
            return;
        }
    };

    send_event(
        event_tx,
        TransportEvent::StateChanged(ConnectionState::Connected),
    );

    // Guest accepts the bidirectional stream opened by host.
    run_connection_io(
        conn,
        &ep,
        false,
        command_rx,
        event_tx,
        reliable_rx,
        unreliable_rx,
        reliable_notify,
        unreliable_notify,
    )
    .await;

    let _ = ep.close().await;
}

/// Run the send/recv I/O loops for an established connection.
///
/// Returns when the connection closes or a Disconnect command is received.
#[allow(clippy::too_many_arguments)]
async fn run_connection_io(
    conn: Connection,
    _ep: &Endpoint,
    is_host: bool,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
    event_tx: &Sender<TransportEvent>,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) {
    // Deterministic stream setup: host opens, guest accepts.
    let (send_stream, recv_stream) = if is_host {
        match conn.open_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                send_error_and_fail(event_tx, format!("Failed to open stream: {e}"));
                return;
            }
        }
    } else {
        match conn.accept_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                send_error_and_fail(event_tx, format!("Failed to accept stream: {e}"));
                return;
            }
        }
    };

    let shutdown = Arc::new(Notify::new());
    let sequence_counter = Arc::new(AtomicU16::new(0));

    // Spawn I/O tasks.
    let send_reliable_handle = tokio::spawn(send_reliable_loop(
        send_stream,
        reliable_rx.clone(),
        reliable_notify.clone(),
        shutdown.clone(),
    ));
    let recv_reliable_handle = tokio::spawn(recv_reliable_loop(
        recv_stream,
        event_tx.clone(),
        shutdown.clone(),
    ));
    let send_unreliable_handle = tokio::spawn(send_unreliable_loop(
        conn.clone(),
        unreliable_rx.clone(),
        sequence_counter,
        unreliable_notify.clone(),
        shutdown.clone(),
    ));
    let recv_unreliable_handle = tokio::spawn(recv_unreliable_loop(
        conn.clone(),
        event_tx.clone(),
        shutdown.clone(),
    ));

    // Wait for disconnect command or connection close.
    tokio::select! {
        _ = wait_for_disconnect(command_rx) => {
            conn.close(0u8.into(), b"disconnect");
        }
        _ = conn.closed() => {
            send_event(event_tx, TransportEvent::Error("Connection lost".into()));
        }
    }

    shutdown.notify_waiters();

    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        let _ = send_reliable_handle.await;
        let _ = recv_reliable_handle.await;
        let _ = send_unreliable_handle.await;
        let _ = recv_unreliable_handle.await;
    })
    .await;

    send_event(
        event_tx,
        TransportEvent::StateChanged(ConnectionState::Disconnected),
    );
}

// ── Reliable I/O ─────────────────────────────────────────────────────────────

/// Send reliable messages from the Bevy channel to the QUIC stream.
/// Woken by `data_notify` when the bridge pushes data.
async fn send_reliable_loop(
    mut send: iroh::endpoint::SendStream,
    reliable_rx: Receiver<Vec<u8>>,
    data_notify: Arc<Notify>,
    shutdown: Arc<Notify>,
) {
    loop {
        // Drain all pending messages first.
        while let Ok(data) = reliable_rx.try_recv() {
            let frame = codec::encode_reliable(&data);
            if let Err(e) = send.write_all(&frame).await {
                warn!("Reliable send error: {e}");
                let _ = send.finish();
                return;
            }
        }

        // Wait for new data or shutdown.
        tokio::select! {
            _ = shutdown.notified() => break,
            _ = data_notify.notified() => {}
        }
    }
    let _ = send.finish();
}

/// Receive reliable messages from the QUIC stream and forward to Bevy.
async fn recv_reliable_loop(
    mut recv: iroh::endpoint::RecvStream,
    event_tx: Sender<TransportEvent>,
    shutdown: Arc<Notify>,
) {
    loop {
        tokio::select! {
            _ = shutdown.notified() => break,
            result = codec::decode_reliable(&mut recv) => {
                match result {
                    Ok(Some(data)) => {
                        send_event(&event_tx, TransportEvent::ReliableMessage(data));
                    }
                    Ok(None) => break,
                    Err(e) => {
                        warn!("Reliable recv error: {e}");
                        break;
                    }
                }
            }
        }
    }
}

// ── Unreliable I/O ───────────────────────────────────────────────────────────

/// Send unreliable data from the Bevy channel as QUIC datagrams.
/// Woken by `data_notify` when the bridge pushes data.
async fn send_unreliable_loop(
    conn: Connection,
    unreliable_rx: Receiver<Vec<u8>>,
    sequence: Arc<AtomicU16>,
    data_notify: Arc<Notify>,
    shutdown: Arc<Notify>,
) {
    // Rate-limit send-error logging — if datagrams are failing every frame
    // we want to know but not flood the log.
    let mut last_send_error_log = std::time::Instant::now()
        .checked_sub(std::time::Duration::from_secs(60))
        .unwrap_or_else(std::time::Instant::now);

    loop {
        // Drain all pending data first.
        while let Ok(data) = unreliable_rx.try_recv() {
            let max_size = conn.max_datagram_size().unwrap_or(DEFAULT_MAX_DATAGRAM);
            let seq = sequence.fetch_add(1, Ordering::Relaxed);
            let fragments = codec::fragment_datagram(&data, seq, max_size);
            for frag in fragments {
                if let Err(e) = conn.send_datagram(frag)
                    && last_send_error_log.elapsed() > std::time::Duration::from_secs(2)
                {
                    warn!("Unreliable datagram send failed (max_size={max_size}): {e}");
                    last_send_error_log = std::time::Instant::now();
                }
            }
        }

        // Wait for new data or shutdown.
        tokio::select! {
            _ = shutdown.notified() => break,
            _ = data_notify.notified() => {}
        }
    }
}

/// Receive QUIC datagrams and reassemble fragments, forwarding complete payloads to Bevy.
async fn recv_unreliable_loop(
    conn: Connection,
    event_tx: Sender<TransportEvent>,
    shutdown: Arc<Notify>,
) {
    let mut reassembler = DatagramReassembler::new();

    loop {
        tokio::select! {
            _ = shutdown.notified() => break,
            result = conn.read_datagram() => {
                match result {
                    Ok(data) => {
                        if let Some(payload) = reassembler.feed(data) {
                            send_event(&event_tx, TransportEvent::UnreliableData(payload));
                        }
                    }
                    Err(e) => {
                        warn!("Datagram recv error: {e}");
                        break;
                    }
                }
            }
        }
    }
}

// ── Connection code encoding/decoding ────────────────────────────────────────

fn encode_endpoint_addr(addr: &EndpointAddr) -> String {
    let bytes = bincode::serialize(addr).expect("EndpointAddr serialization should not fail");
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

fn decode_endpoint_addr(code: &str) -> Result<EndpointAddr, String> {
    let bytes = data_encoding::BASE64URL_NOPAD
        .decode(code.as_bytes())
        .map_err(|e| format!("Invalid base64: {e}"))?;
    bincode::deserialize(&bytes).map_err(|e| format!("Invalid address data: {e}"))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn send_event(tx: &Sender<TransportEvent>, event: TransportEvent) {
    let _ = tx.send(event);
}

/// Send an error message and transition to Failed state.
fn send_error_and_fail(tx: &Sender<TransportEvent>, msg: String) {
    send_event(tx, TransportEvent::Error(msg));
    send_event(tx, TransportEvent::StateChanged(ConnectionState::Failed));
}

/// Async wait for a Disconnect command on the tokio mpsc channel.
async fn wait_for_disconnect(
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
) {
    loop {
        match command_rx.recv().await {
            Some(TransportCommand::Disconnect) | None => return,
            Some(_) => continue, // Ignore non-disconnect commands during active connection.
        }
    }
}
