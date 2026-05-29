//! Bevy plugin that bridges the async transport runtime with the ECS.

use std::sync::Arc;

use bevy::prelude::*;
use crossbeam_channel::unbounded;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, NetworkConnection};

use super::connection;
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

    // When the active mode is Steam, the Steam transport owns the message
    // queues — but we still drain `event_rx` (and discard) so that stale
    // events from a prior iroh session (e.g. a `StateChanged(Disconnected)`
    // or `Error("Connection lost")` emitted by the tokio teardown AFTER the
    // user clicked Cancel and immediately started Steam) don't pile up and
    // then overwrite a clean state when `connection.reset()` later flips
    // `mode` back to `Online`.
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

    // Receive transport events.
    while let Ok(event) = handle.event_rx.try_recv() {
        match event {
            TransportEvent::StateChanged(state) => {
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
