//! Tokio runtime management and channel types for the transport layer.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use crate::networking::resources::ConnectionState;

/// Commands sent from Bevy systems to the transport runtime.
#[derive(Debug)]
pub(crate) enum TransportCommand {
    /// Create a host endpoint and start listening for a guest connection.
    CreateHost {
        /// Whether to use relay servers for NAT traversal.
        use_relay: bool,
    },

    /// Connect to a host using their ticket code.
    ConnectToHost {
        /// The base64url-encoded endpoint address from the host.
        ticket_code: String,
    },

    /// Disconnect and tear down the current connection.
    Disconnect,
}

/// Events sent from the transport runtime back to Bevy systems.
#[derive(Debug)]
pub(super) enum TransportEvent {
    /// Connection state changed.
    StateChanged(ConnectionState),

    /// A local connection code is ready for the user to copy.
    LocalCode(String),

    /// A reliable message was received from the remote peer (bincode-serialized `NetworkMessage`).
    ReliableMessage(Vec<u8>),

    /// Unreliable data received from the remote peer (raw binary snapshot).
    UnreliableData(Vec<u8>),

    /// Updated ping measurement in milliseconds.
    #[allow(dead_code)]
    PingUpdate(f32),

    /// An error occurred in the transport layer.
    Error(String),
}

/// A generation-stamped view of the tokio → Bevy event channel.
///
/// Every event carries the generation of the command that started the flow emitting
/// it, so the Bevy side can tell "this session" from "the one we just tore down".
/// Without it, teardown events emitted *after* `reset_multiplayer_to_baseline`
/// returned — `Error("Connection lost")`, a late `StateChanged`, or a `LocalCode` for
/// an endpoint that is already closed — landed on the fresh connection and either
/// painted a spurious "Connection lost" panel or handed the host a dead code to share.
#[derive(Clone)]
pub(super) struct EventSink {
    tx: Sender<(u64, TransportEvent)>,
    generation: u64,
}

impl EventSink {
    pub(super) fn new(tx: Sender<(u64, TransportEvent)>, generation: u64) -> Self {
        Self { tx, generation }
    }

    pub(super) fn send(&self, event: TransportEvent) {
        let _ = self.tx.send((self.generation, event));
    }

    /// A sink that stamps for a *different* generation.
    ///
    /// Used by `wait_for_disconnect` when it has to reject a newer `CreateHost` /
    /// `ConnectToHost` that arrived mid-teardown: that rejection is news about the
    /// NEW command, so stamping it with the dying flow's generation would get it
    /// fenced out and the player would click Host Game to no visible effect.
    pub(super) fn at_generation(&self, generation: u64) -> Self {
        Self {
            tx: self.tx.clone(),
            generation,
        }
    }
}

/// Bevy resource holding the channel handles for communicating with the transport runtime.
///
/// Inserted by `TransportBridgePlugin` on startup. Game systems send commands and
/// the bridge system drains events each frame.
#[derive(Resource)]
pub(crate) struct TransportHandle {
    /// Send commands to the transport runtime (create host, connect, disconnect),
    /// each stamped with the session generation it belongs to.
    pub(super) command_tx: tokio::sync::mpsc::UnboundedSender<(u64, TransportCommand)>,

    /// Receive generation-stamped events from the transport runtime.
    pub(super) event_rx: Receiver<(u64, TransportEvent)>,

    /// Bumped by every command. Events stamped with anything else are stale and
    /// belong to a session the Bevy world has already torn down.
    pub(super) session_gen: AtomicU64,

    /// Send reliable messages to the remote peer (bincode-serialized `NetworkMessage`).
    pub(super) reliable_tx: crossbeam_channel::Sender<Vec<u8>>,

    /// Send unreliable data to the remote peer (raw binary snapshots).
    pub(super) unreliable_tx: crossbeam_channel::Sender<Vec<u8>>,

    /// Wakes the reliable send loop when outgoing reliable data is available.
    ///
    /// Each send loop has its own notifier — a single shared `Notify` with
    /// `notify_one()` would wake only one of the two loops, silently stranding
    /// messages destined for the other.
    pub(super) reliable_notify: Arc<tokio::sync::Notify>,

    /// Wakes the unreliable send loop when outgoing unreliable data is available.
    pub(super) unreliable_notify: Arc<tokio::sync::Notify>,
}

impl TransportHandle {
    /// Send a command to the transport runtime, opening a new session generation.
    ///
    /// `Disconnect` bumps too, and deliberately so: that is what makes the dying
    /// flow's own teardown events stale the instant the Bevy world stops caring
    /// about them.
    pub(crate) fn send_command(&self, cmd: TransportCommand) {
        let generation = self.session_gen.fetch_add(1, Ordering::SeqCst) + 1;
        if let Err(e) = self.command_tx.send((generation, cmd)) {
            warn!("Failed to send transport command: {e}");
        }
    }

    /// The generation the Bevy world currently considers live.
    pub(super) fn current_generation(&self) -> u64 {
        self.session_gen.load(Ordering::SeqCst)
    }

    /// Throw away everything the transport has queued for us.
    ///
    /// Called from `reset_multiplayer_to_baseline`. The generation fence already
    /// stops stale events being *applied*; this stops them accumulating, and keeps
    /// the reset honest if the fence is ever bypassed.
    pub(crate) fn drain_events(&self) {
        while self.event_rx.try_recv().is_ok() {}
    }
}
