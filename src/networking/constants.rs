//! Networking constants.

/// Google's free public STUN server for NAT traversal discovery.
pub(super) const STUN_URL: &str = "stun:stun.l.google.com:19302";

/// Name of the reliable WebRTC data channel used for game commands and events.
pub(super) const DATA_CHANNEL_NAME: &str = "game";

/// Name of the unreliable WebRTC data channel used for state snapshots.
pub(super) const UNRELIABLE_CHANNEL_NAME: &str = "game_unreliable";

/// Interval between ping messages in seconds.
pub(super) const PING_INTERVAL_SECS: f32 = 2.0;

/// Format version for the compact binary connection code.
pub(super) const CONNECTION_CODE_VERSION: u8 = 1;

/// Name of the BroadcastChannel used for same-machine LAN signaling.
pub(super) const LAN_BROADCAST_CHANNEL: &str = "court_wizard_lan";
