//! Networking resources.

use bevy::prelude::*;

use super::protocol::NetworkMessage;

/// Whether the connection uses internet (STUN) or LAN (no STUN).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionMode {
    /// Online mode: uses STUN server for NAT traversal.
    #[default]
    Online,

    /// LAN mode: no STUN, host candidates only. Works without internet.
    Lan,
}

/// The current state of the network connection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    /// No connection active.
    #[default]
    Disconnected,

    /// Waiting for the user to exchange signaling codes.
    WaitingForSignaling,

    /// SDP exchange complete, waiting for data channel to open.
    Connecting,

    /// Data channel is open and messages can be sent/received.
    Connected,

    /// Connection failed or was lost.
    Failed,
}

/// Whether this peer is the host or guest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerRole {
    /// Created the offer (Player A).
    Host,

    /// Received the offer and created the answer (Player B).
    Guest,
}

/// Global resource tracking the P2P network connection.
#[derive(Resource, Default)]
pub struct NetworkConnection {
    /// Current connection state.
    pub state: ConnectionState,

    /// Role of this peer in the connection.
    pub role: Option<PeerRole>,

    /// Base64-encoded SDP+ICE code to display to the user for copy-paste signaling.
    pub local_code: Option<String>,

    /// Reliable messages received from the remote peer, drained each frame by game systems.
    pub incoming_messages: Vec<NetworkMessage>,

    /// Reliable messages queued to send to the remote peer.
    pub outgoing_messages: Vec<NetworkMessage>,

    /// Raw binary data received on the unreliable channel (state snapshots).
    pub incoming_unreliable: Vec<Vec<u8>>,

    /// Raw binary data queued to send on the unreliable channel.
    pub outgoing_unreliable: Vec<Vec<u8>>,

    /// Latest measured round-trip ping in milliseconds.
    pub ping_ms: Option<f32>,

    /// Timer tracking when to send the next ping.
    pub ping_timer: f32,

    /// Error message if connection failed.
    pub error: Option<String>,

    /// Connection mode (Online with STUN, or LAN without).
    pub mode: ConnectionMode,
}
