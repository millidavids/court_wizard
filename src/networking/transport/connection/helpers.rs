//! Shared async/sync helpers: event sending, error reporting, disconnect wait.

use crossbeam_channel::Sender;

use crate::networking::resources::ConnectionState;

use crate::networking::transport::runtime::{TransportCommand, TransportEvent};

pub(super) fn send_event(tx: &Sender<TransportEvent>, event: TransportEvent) {
    let _ = tx.send(event);
}

/// Send an error message and transition to Failed state.
pub(super) fn send_error_and_fail(tx: &Sender<TransportEvent>, msg: String) {
    send_event(tx, TransportEvent::Error(msg));
    send_event(tx, TransportEvent::StateChanged(ConnectionState::Failed));
}

/// Async wait for a Disconnect command on the tokio mpsc channel.
pub(super) async fn wait_for_disconnect(
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<TransportCommand>,
) {
    loop {
        match command_rx.recv().await {
            Some(TransportCommand::Disconnect) | None => return,
            Some(_) => continue, // Ignore non-disconnect commands during active connection.
        }
    }
}
