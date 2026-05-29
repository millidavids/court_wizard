//! Constants for the Steam multiplayer transport.

/// Virtual port used for both the host's listen socket and the guest's
/// `connect_p2p`. Per-app value; any non-negative i32 is fine.
pub(super) const VIRTUAL_PORT: i32 = 0;

/// Key under which we stamp `PROTOCOL_VERSION` in lobby metadata so the guest
/// can fast-reject incompatible hosts before opening a socket.
pub(super) const LOBBY_KEY_PROTOCOL_VERSION: &str = "cwproto";

/// Rich-presence key Steam looks at to enable the friend-list "Join Game"
/// option.
pub(super) const RICH_PRESENCE_CONNECT_KEY: &str = "connect";

/// How many `NetworkingMessage`s to pull off the wire per frame. Steam's docs
/// recommend draining up to ~256 per call; 64 is plenty for our message rate.
pub(super) const RECEIVE_BATCH: usize = 64;

/// Channel tag prepended to reliable payloads on the wire.
pub(super) const TAG_RELIABLE: u8 = 0x01;

/// Channel tag prepended to unreliable payloads on the wire.
pub(super) const TAG_UNRELIABLE: u8 = 0x02;
