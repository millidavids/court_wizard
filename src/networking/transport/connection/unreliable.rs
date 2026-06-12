//! Unreliable I/O loops — send/receive QUIC datagrams with fragmentation/reassembly.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use crossbeam_channel::{Receiver, Sender};
use iroh::endpoint::Connection;
use tokio::sync::Notify;
use tracing::warn;

use super::endpoint::DEFAULT_MAX_DATAGRAM;
use super::helpers::send_event;
use crate::networking::transport::codec::{self, DatagramReassembler};
use crate::networking::transport::runtime::TransportEvent;

/// Send unreliable data from the Bevy channel as QUIC datagrams.
/// Woken by `data_notify` when the bridge pushes data.
pub(super) async fn send_unreliable_loop(
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
pub(super) async fn recv_unreliable_loop(
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
