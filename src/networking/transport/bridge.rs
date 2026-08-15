//! Bevy plugin that bridges the async transport runtime with the ECS.

use std::sync::Arc;

use bevy::prelude::*;
use crossbeam_channel::unbounded;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection};

use super::connection;
#[cfg(test)]
use super::runtime::TransportCommand;
use super::runtime::{TransportEvent, TransportHandle};

/// Plugin that manages the P2P transport layer.
///
/// Spawns a tokio runtime in a background thread and registers a bridge system
/// that syncs the async transport state with `NetworkConnection` each frame.
pub(crate) struct TransportBridgePlugin;

impl Plugin for TransportBridgePlugin {
    fn build(&self, app: &mut App) {
        // Command channel: Bevy → tokio (async-native, no blocking recv needed).
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

        // Event channel: tokio → Bevy (crossbeam for sync try_recv on the Bevy side).
        let (event_tx, event_rx) = unbounded();

        // Data channels: Bevy → tokio (crossbeam, woken by Notify).
        let (reliable_tx, reliable_rx) = unbounded();
        let (unreliable_tx, unreliable_rx) = unbounded();

        // One notify per send loop. A single shared notify with `notify_one()`
        // would wake only one of the two loops, stranding the other's messages.
        let reliable_notify = Arc::new(tokio::sync::Notify::new());
        let unreliable_notify = Arc::new(tokio::sync::Notify::new());
        let reliable_notify_rt = reliable_notify.clone();
        let unreliable_notify_rt = unreliable_notify.clone();

        // Spawn tokio runtime in a background thread.
        std::thread::Builder::new()
            .name("transport-runtime".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime for transport");

                rt.block_on(connection::run_transport(
                    command_rx,
                    event_tx,
                    reliable_rx,
                    unreliable_rx,
                    reliable_notify_rt,
                    unreliable_notify_rt,
                ));

                // Iroh spawns long-lived actor tasks (relay/STUN probes, pkarr
                // publisher, mDNS) inside our runtime. The default `Runtime`
                // drop blocks until every task finishes, which on macOS can
                // stall for tens of seconds at app exit because those network
                // tasks never see clean shutdown. `shutdown_background` aborts
                // all tasks and returns immediately so the OS thread exits.
                rt.shutdown_background();
            })
            .expect("Failed to spawn transport runtime thread");

        app.insert_resource(TransportHandle {
            command_tx,
            event_rx,
            session_gen: std::sync::atomic::AtomicU64::new(0),
            reliable_tx,
            unreliable_tx,
            reliable_notify,
            unreliable_notify,
        });

        app.add_systems(PreUpdate, transport_bridge_system);
    }
}

/// Bridge system: drains outgoing queues from `NetworkConnection` into the transport,
/// and receives events from the transport to populate incoming queues and update state.
///
/// Uses read-only access first to avoid triggering Bevy change detection when idle.
fn transport_bridge_system(
    handle: Option<Res<TransportHandle>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let Some(handle) = handle else { return };

    // When the active mode is Steam, the Steam transport owns the message queues.
    //
    // This early `return` is load-bearing for far more than the drain: it is what
    // stops THIS system draining `outgoing_messages` / `outgoing_unreliable` into the
    // (dead) iroh channels in `PreUpdate`, before `steam_transport_bridge_system` gets
    // to look at them in `Update`. Remove it and all Steam P2P traffic silently stops.
    //
    // The drain itself is now belt-and-braces — the generation fence below is the real
    // guard against stale iroh events — but it still keeps the queue from growing while
    // a Steam session runs.
    if connection.mode == ConnectionMode::Steam {
        while handle.event_rx.try_recv().is_ok() {}
        return;
    }

    // Check if there's any work via immutable access (doesn't trigger change detection).
    let has_outgoing =
        !connection.outgoing_messages.is_empty() || !connection.outgoing_unreliable.is_empty();
    let has_incoming = !handle.event_rx.is_empty();

    if !has_outgoing && !has_incoming {
        return;
    }

    // Never queue bytes for a link that is down. The transport channels are
    // process-lifetime and shared across sessions, so anything pushed while
    // disconnected would be delivered to the NEXT peer ahead of its
    // `HandshakeVersion` — which the receiver rejects as a version mismatch.
    // (`run_connection_io` also drains at session start; this stops the queue
    // growing in the first place.)
    //
    // Deliberately NOT `!= Connected`: this system is itself what promotes `state`
    // to `Connected` from the event queue below, so a stricter gate would drop
    // everything queued during the `Connecting → Connected` window.
    if matches!(
        connection.state,
        ConnectionState::Disconnected | ConnectionState::Failed
    ) {
        connection.outgoing_messages.clear();
        connection.outgoing_unreliable.clear();
    }

    // Send outgoing reliable messages.
    let mut sent_reliable = false;
    let mut sent_unreliable = false;
    for msg in connection.outgoing_messages.drain(..) {
        match bincode::serialize(&msg) {
            Ok(data) => {
                let _ = handle.reliable_tx.send(data);
                sent_reliable = true;
            }
            Err(e) => {
                warn!("Failed to serialize outgoing message: {e}");
            }
        }
    }

    // Send outgoing unreliable data.
    for data in connection.outgoing_unreliable.drain(..) {
        let _ = handle.unreliable_tx.send(data);
        sent_unreliable = true;
    }

    // Wake whichever send loop(s) we actually pushed data to.
    if sent_reliable {
        handle.reliable_notify.notify_one();
    }
    if sent_unreliable {
        handle.unreliable_notify.notify_one();
    }

    // Receive transport events, discarding anything belonging to a session the world
    // has already torn down. The tokio side keeps emitting for a while after
    // `reset_multiplayer_to_baseline` returns — a trailing `Error("Connection lost")`,
    // a `StateChanged`, or a `LocalCode` for an endpoint that is already closing — and
    // applying those to the fresh connection is what produced a spurious
    // "Connection lost" panel or a dead code in the host's panel.
    let current_generation = handle.current_generation();
    while let Ok((generation, event)) = handle.event_rx.try_recv() {
        if generation != current_generation {
            continue;
        }
        match event {
            TransportEvent::StateChanged(state) => {
                // A successful connect retires whatever error the previous attempt
                // left behind. `connection.error` is otherwise only cleared by
                // `reset()`, so a stale message could caption an unrelated later
                // failure (`sync_lobby_with_connection` reads it verbatim).
                if state == ConnectionState::Connected {
                    connection.error = None;
                }
                connection.state = state;
            }
            TransportEvent::LocalCode(code) => {
                connection.local_code = Some(code);
            }
            TransportEvent::ReliableMessage(data) => {
                match bincode::deserialize::<NetworkMessage>(&data) {
                    Ok(msg) => {
                        connection.incoming_messages.push(msg);
                    }
                    Err(e) => {
                        warn!("Failed to deserialize incoming message: {e}");
                    }
                }
            }
            TransportEvent::UnreliableData(data) => {
                connection.incoming_unreliable.push(data);
            }
            TransportEvent::PingUpdate(ms) => {
                connection.ping_ms = Some(ms);
            }
            TransportEvent::Error(msg) => {
                error!("Transport error: {msg}");
                connection.error = Some(msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crossbeam_channel::{Receiver, Sender};
    use std::sync::atomic::AtomicU64;

    /// Channel ends the handle doesn't own but that must outlive it, or the
    /// crossbeam/mpsc senders report "disconnected" and the test stops exercising
    /// the real code path.
    struct Keep {
        _command_rx: tokio::sync::mpsc::UnboundedReceiver<(u64, TransportCommand)>,
        _reliable_rx: Receiver<Vec<u8>>,
        _unreliable_rx: Receiver<Vec<u8>>,
    }

    fn handle_at(generation: u64) -> (TransportHandle, Sender<(u64, TransportEvent)>, Keep) {
        let (command_tx, _command_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = unbounded();
        let (reliable_tx, _reliable_rx) = unbounded();
        let (unreliable_tx, _unreliable_rx) = unbounded();
        (
            TransportHandle {
                command_tx,
                event_rx,
                session_gen: AtomicU64::new(generation),
                reliable_tx,
                unreliable_tx,
                reliable_notify: Arc::new(tokio::sync::Notify::new()),
                unreliable_notify: Arc::new(tokio::sync::Notify::new()),
            },
            event_tx,
            Keep {
                _command_rx,
                _reliable_rx,
                _unreliable_rx,
            },
        )
    }

    /// The regression that would silently re-break reconnecting: the tokio side keeps
    /// emitting for a while after the Bevy world has torn a session down, and those
    /// trailing events must not land on the fresh one.
    #[test]
    fn events_from_a_torn_down_session_are_discarded() {
        let (handle, event_tx, _keep) = handle_at(2);

        // Session 1's teardown, emitted after the world already reset.
        let _ = event_tx.send((1, TransportEvent::Error("Connection lost".into())));
        let _ = event_tx.send((1, TransportEvent::StateChanged(ConnectionState::Failed)));
        let _ = event_tx.send((1, TransportEvent::LocalCode("dead-code".into())));
        // Session 2, live.
        let _ = event_tx.send((2, TransportEvent::LocalCode("live-code".into())));
        let _ = event_tx.send((2, TransportEvent::StateChanged(ConnectionState::Connected)));

        let mut app = App::new();
        app.insert_resource(NetworkConnection::default());
        app.insert_resource(handle);
        app.world_mut()
            .run_system_once(transport_bridge_system)
            .expect("bridge system should run");

        let connection = app.world().resource::<NetworkConnection>();
        assert_eq!(connection.state, ConnectionState::Connected);
        assert_eq!(
            connection.error, None,
            "a stale error must not caption the live session"
        );
        assert_eq!(
            connection.local_code.as_deref(),
            Some("live-code"),
            "a stale LocalCode must not replace the live one — the host would share a \
             code for an endpoint that is already closed"
        );
    }

    /// The fence must not swallow the CURRENT session's failures.
    #[test]
    fn current_generation_events_still_apply() {
        let (handle, event_tx, _keep) = handle_at(7);
        let _ = event_tx.send((7, TransportEvent::Error("real failure".into())));
        let _ = event_tx.send((7, TransportEvent::StateChanged(ConnectionState::Failed)));

        let mut app = App::new();
        app.insert_resource(NetworkConnection::default());
        app.insert_resource(handle);
        app.world_mut()
            .run_system_once(transport_bridge_system)
            .expect("bridge system should run");

        let connection = app.world().resource::<NetworkConnection>();
        assert_eq!(connection.state, ConnectionState::Failed);
        assert_eq!(connection.error.as_deref(), Some("real failure"));
    }

    /// `send_command` opens a new generation, which is what makes the dying flow's
    /// events stale, and `drain_events` clears what is already queued.
    #[test]
    fn send_command_bumps_the_generation_and_drain_empties_the_queue() {
        let (handle, event_tx, _keep) = handle_at(0);
        let _ = event_tx.send((0, TransportEvent::Error("stale".into())));

        handle.send_command(TransportCommand::Disconnect);
        assert_eq!(handle.current_generation(), 1);

        handle.drain_events();
        assert!(handle.event_rx.is_empty());
    }
}
