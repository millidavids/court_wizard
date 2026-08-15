//! Reliable I/O loops — send/receive over the QUIC bidirectional stream.

use std::sync::Arc;

use bevy::log::warn;
use crossbeam_channel::Receiver;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use super::helpers::send_event;
use crate::networking::transport::codec;
use crate::networking::transport::runtime::{EventSink, TransportEvent};

/// Send reliable messages from the Bevy channel to the QUIC stream.
/// Woken by `data_notify` when the bridge pushes data.
pub(super) async fn send_reliable_loop(
    mut send: iroh::endpoint::SendStream,
    reliable_rx: Receiver<Vec<u8>>,
    data_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    loop {
        // Re-check before draining: `shutdown` is sticky, so a cancel that landed
        // while we were mid-`write_all` is still visible here. Without this a
        // cancelled loop would drain one more batch belonging to the next session.
        if shutdown.is_cancelled() {
            break;
        }

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
            _ = shutdown.cancelled() => break,
            _ = data_notify.notified() => {}
        }
    }
    let _ = send.finish();
}

/// Receive reliable messages from the QUIC stream and forward to Bevy.
pub(super) async fn recv_reliable_loop(
    mut recv: iroh::endpoint::RecvStream,
    events: EventSink,
    shutdown: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = codec::decode_reliable(&mut recv) => {
                match result {
                    Ok(Some(data)) => {
                        send_event(&events, TransportEvent::ReliableMessage(data));
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
