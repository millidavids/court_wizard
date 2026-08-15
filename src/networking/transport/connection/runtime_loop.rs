//! Transport runtime entry point — the main async loop that processes commands from Bevy.

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use tokio::sync::Notify;

use super::guest::handle_guest;
use super::host::handle_host;
use crate::networking::transport::runtime::{EventSink, TransportCommand, TransportEvent};

/// Main entry point for the transport runtime. Runs on the tokio thread and
/// processes commands from the Bevy bridge.
///
/// Each command carries the session generation the Bevy side opened for it; every
/// event the resulting flow emits is stamped with that generation so the bridge can
/// fence out a session the world has already torn down.
pub(crate) async fn run_transport(
    mut command_rx: tokio::sync::mpsc::UnboundedReceiver<(u64, TransportCommand)>,
    event_tx: Sender<(u64, TransportEvent)>,
    reliable_rx: Receiver<Vec<u8>>,
    unreliable_rx: Receiver<Vec<u8>>,
    reliable_notify: Arc<Notify>,
    unreliable_notify: Arc<Notify>,
) {
    loop {
        let (generation, cmd) = match command_rx.recv().await {
            Some(cmd) => cmd,
            None => return, // Channel closed — Bevy is shutting down.
        };

        let sink = EventSink::new(event_tx.clone(), generation);

        match cmd {
            TransportCommand::CreateHost { use_relay } => {
                handle_host(
                    use_relay,
                    &mut command_rx,
                    &sink,
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
                    &sink,
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
