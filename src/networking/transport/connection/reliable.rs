//! Reliable I/O loops — send/receive over the QUIC bidirectional stream.

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use tokio::sync::Notify;
use tracing::warn;

use super::helpers::send_event;
use crate::networking::transport::codec;
use crate::networking::transport::runtime::TransportEvent;

/// Send reliable messages from the Bevy channel to the QUIC stream.
/// Woken by `data_notify` when the bridge pushes data.
pub(super) async fn send_reliable_loop(
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
pub(super) async fn recv_reliable_loop(
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
