//! Shared async/sync helpers: event sending, error reporting, disconnect wait.

use crate::networking::resources::ConnectionState;

use crate::networking::transport::runtime::{EventSink, TransportCommand, TransportEvent};

pub(super) fn send_event(sink: &EventSink, event: TransportEvent) {
    sink.send(event);
}

/// Send an error message and transition to Failed state.
pub(super) fn send_error_and_fail(sink: &EventSink, msg: String) {
    send_event(sink, TransportEvent::Error(msg));
    send_event(sink, TransportEvent::StateChanged(ConnectionState::Failed));
}

/// Async wait for a Disconnect command on the tokio mpsc channel.
///
/// A `CreateHost`/`ConnectToHost` arriving here means the UI asked to start a new
/// session while this one is still winding down. The runtime is a single serial
/// loop, so it cannot honour that — but dropping it silently (which is what this
/// used to do) makes the button look broken: the player clicks Host Game and
/// literally nothing happens, forever. Surface it so the lobby shows a Failed
/// panel with a Retry that will work.
///
/// The rejection is stamped with the **incoming** command's generation, not this
/// flow's. It is news about the new command, and the bridge fences out anything
/// stamped with a generation the world has moved past — so stamping it with the
/// dying flow's generation would discard it and reinstate exactly the silent
/// dead-button failure described above.
pub(super) async fn wait_for_disconnect(
    command_rx: &mut tokio::sync::mpsc::UnboundedReceiver<(u64, TransportCommand)>,
    sink: &EventSink,
) {
    loop {
        match command_rx.recv().await {
            Some((_, TransportCommand::Disconnect)) | None => return,
            Some((generation, other)) => {
                send_error_and_fail(
                    &sink.at_generation(generation),
                    format!(
                        "A previous connection was still closing ({other:?} ignored). Try again."
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::networking::transport::runtime::EventSink;

    /// The rejection of a command that arrived mid-teardown must be stamped with
    /// THAT command's generation.
    ///
    /// Stamp it with the dying flow's generation instead and `transport_bridge_system`
    /// fences it out — the player clicks Host Game, the runtime eats the command, the
    /// error never surfaces, and the button appears dead forever. That is exactly the
    /// failure this function was written to prevent, so it is worth a test.
    #[test]
    fn a_command_eaten_mid_teardown_is_reported_at_its_own_generation() {
        let (command_tx, mut command_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        // This flow belongs to generation 1. The player then clicks Host Game
        // (generation 2) before it has finished winding down, and finally something
        // issues the Disconnect (generation 3) that lets us return.
        let sink = EventSink::new(event_tx, 1);
        command_tx
            .send((2, TransportCommand::CreateHost { use_relay: true }))
            .expect("send CreateHost");
        command_tx
            .send((3, TransportCommand::Disconnect))
            .expect("send Disconnect");

        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("build test runtime")
            .block_on(wait_for_disconnect(&mut command_rx, &sink));

        let events: Vec<(u64, TransportEvent)> = event_rx.try_iter().collect();
        assert!(
            !events.is_empty(),
            "the rejected command must surface as an error, not vanish"
        );
        assert!(
            events.iter().all(|(generation, _)| *generation == 2),
            "expected every event stamped with the incoming command's generation (2), got {:?}",
            events.iter().map(|(g, _)| *g).collect::<Vec<_>>()
        );
    }
}
