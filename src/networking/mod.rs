//! Networking module for P2P WebRTC communication.
//!
//! Provides serverless multiplayer connectivity using WebRTC with
//! copy-paste SDP signaling (the human is the signaling channel).

pub(crate) mod crdt;
pub(crate) mod entity_map;
mod messages;
mod plugin;
pub(crate) mod protocol;
pub(crate) mod resources;
pub(crate) mod session;
pub(crate) mod snapshot;

pub use plugin::NetworkingPlugin;
