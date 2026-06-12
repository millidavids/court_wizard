//! Transport runtime entry point — the main async loop that processes commands from Bevy.

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use tokio::sync::Notify;

use super::guest::handle_guest;
use super::host::handle_host;
use crate::networking::transport::runtime::{TransportCommand, TransportEvent};

/// Main entry point for the transport runtime. Runs on the tokio thread and
/// processes commands from the Bevy bridge.
pub(crate) async fn run_transport(
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
