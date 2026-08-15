//! Connection I/O orchestration — stream setup, I/O task spawning, disconnect handling.

use std::sync::Arc;
use std::sync::atomic::AtomicU16;

use bevy::log::warn;
use crossbeam_channel::Receiver;
use iroh::Endpoint;
use iroh::endpoint::Connection;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::networking::resources::ConnectionState;
use crate::networking::transport::runtime::{EventSink, TransportCommand, TransportEvent};

use super::helpers::{send_error_and_fail, send_event, wait_for_disconnect};
use super::reliable::{recv_reliable_loop, send_reliable_loop};
use super::unreliable::{recv_unreliable_loop, send_unreliable_loop};

/// Why `run_connection_io` returned. The host uses this to decide whether to
/// re-listen for a reconnection (`Lost`) or shut the endpoint down (`Disconnect`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConnectionExitReason {
    /// An explicit `TransportCommand::Disconnect` arrived — the local peer is
    /// leaving for good. Never re-listen.
    Disconnect,
    /// The connection dropped without an explicit leave (peer vanished). The
    /// co-op host re-listens so the same guest can reconnect.
    Lost,
}

/// Run the send/recv I/O loops for an established connection.
///
/// Returns when the connection closes or a Disconnect command is received.
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_connection_io(
    conn: Connection,
    _ep: &Endpoint,
    is_host: bool,
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<(u64, TransportCommand)>,
    events: &EventSink,
    reliable_rx: &Receiver<Vec<u8>>,
    unreliable_rx: &Receiver<Vec<u8>>,
    reliable_notify: &Arc<Notify>,
    unreliable_notify: &Arc<Notify>,
) -> ConnectionExitReason {
    // Discard anything left in the outgoing queues from a PREVIOUS session.
    //
    // The queues are process-lifetime unbounded crossbeam channels shared by every
    // session, and `transport_bridge_system` pushes into them from the Bevy side.
    // The per-session send loops exit at teardown leaving whatever they hadn't sent
    // still queued — which then flushes to the NEXT peer, ahead of the
    // `HandshakeVersion` frame. The receiver enforces "HandshakeVersion must come
    // first" and would reject a perfectly good connection as a version mismatch.
    //
    // MUST be the first statement, before any `.await`. Our callers emit
    // `StateChanged(Connected)` immediately before calling us, and once Bevy
    // observes that it starts pushing this session's `HandshakeVersion` +
    // `PlayerInfo`. Draining after a yield point (e.g. below the stream setup)
    // would race that and eat the handshake — reintroducing the exact deadlock
    // this drain exists to prevent. With no await between the event and here, the
    // window is nanoseconds against Bevy's frame-plus-a-system.
    let stale = reliable_rx.len() + unreliable_rx.len();
    if stale > 0 {
        warn!("[Transport] Discarding {stale} queued message(s) from a previous session");
        while reliable_rx.try_recv().is_ok() {}
        while unreliable_rx.try_recv().is_ok() {}
    }

    // Deterministic stream setup: host opens, guest accepts.
    let (send_stream, recv_stream) = if is_host {
        match conn.open_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                send_error_and_fail(events, format!("Failed to open stream: {e}"));
                return ConnectionExitReason::Lost;
            }
        }
    } else {
        match conn.accept_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                send_error_and_fail(events, format!("Failed to accept stream: {e}"));
                return ConnectionExitReason::Lost;
            }
        }
    };

    // Per-session cancellation. This MUST NOT be a `Notify`: `notify_waiters()`
    // stores no permit, and tokio discards a pending `Notification::All` when the
    // `Notified` future is dropped. Both send loops sit in a two-branch `select!`
    // (shutdown vs. data), and `select!` polls its branches in random order — so a
    // loop woken by the data branch on the same poll that shutdown fired would drop
    // the shutdown notification and park forever on the next iteration. It then
    // outlived the session still holding a clone of the process-lifetime MPMC
    // `reliable_rx`/`unreliable_rx`, and the NEXT session's `notify_one()` handed it
    // that session's bytes — which is how a reconnect lost its `HandshakeVersion` +
    // `PlayerInfo` (hard hang) or a share of its snapshots (worsening stutter).
    // `CancellationToken` is level-triggered and sticky, so it cannot be missed.
    let shutdown = CancellationToken::new();
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
        events.clone(),
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
        events.clone(),
        shutdown.clone(),
    ));

    // Wait for disconnect command or connection close.
    let reason = tokio::select! {
        _ = wait_for_disconnect(command_rx, events) => {
            conn.close(0u8.into(), b"disconnect");
            ConnectionExitReason::Disconnect
        }
        _ = conn.closed() => {
            send_event(events, TransportEvent::Error("Connection lost".into()));
            ConnectionExitReason::Lost
        }
    };

    shutdown.cancel();

    // Give the loops a bounded window to finish in place, then ABORT unconditionally.
    //
    // Awaiting by `&mut handle` (`JoinHandle` is `Unpin`) keeps the handles alive past
    // the timeout so we can still abort them. The previous version moved them into the
    // async block, so a timeout DROPPED them — and dropping a `JoinHandle` detaches the
    // task rather than aborting it, which is what let a stuck loop survive into the next
    // session. Aborting an already-finished task is a no-op, so this costs nothing on the
    // happy path and is deterministic on every other one.
    let mut handles = [
        send_reliable_handle,
        recv_reliable_handle,
        send_unreliable_handle,
        recv_unreliable_handle,
    ];
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        for handle in &mut handles {
            let _ = handle.await;
        }
    })
    .await;
    for handle in &handles {
        handle.abort();
    }

    send_event(
        events,
        TransportEvent::StateChanged(ConnectionState::Disconnected),
    );

    reason
}
