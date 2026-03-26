//! Tokio runtime management and channel types for the transport layer.

use std::sync::Arc;

use bevy::prelude::*;
use crossbeam_channel::Receiver;

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

/// Bevy resource holding the channel handles for communicating with the transport runtime.
///
/// Inserted by `TransportBridgePlugin` on startup. Game systems send commands and
/// the bridge system drains events each frame.
#[derive(Resource)]
pub(crate) struct TransportHandle {
    /// Send commands to the transport runtime (create host, connect, disconnect).
    pub(super) command_tx: tokio::sync::mpsc::UnboundedSender<TransportCommand>,

    /// Receive events from the transport runtime (state changes, messages, errors).
    pub(super) event_rx: Receiver<TransportEvent>,

    /// Send reliable messages to the remote peer (bincode-serialized `NetworkMessage`).
    pub(super) reliable_tx: crossbeam_channel::Sender<Vec<u8>>,

    /// Send unreliable data to the remote peer (raw binary snapshots).
    pub(super) unreliable_tx: crossbeam_channel::Sender<Vec<u8>>,

    /// Wakes the async send loops when outgoing data is available.
    pub(super) data_notify: Arc<tokio::sync::Notify>,
}

impl TransportHandle {
    /// Send a command to the transport runtime.
    pub(crate) fn send_command(&self, cmd: TransportCommand) {
        if let Err(e) = self.command_tx.send(cmd) {
            warn!("Failed to send transport command: {e}");
        }
    }
}
