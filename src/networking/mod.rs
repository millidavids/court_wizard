//! Networking module for P2P WebRTC communication.
//!
//! Provides serverless multiplayer connectivity using WebRTC with
//! copy-paste SDP signaling (the human is the signaling channel).

#[cfg(target_arch = "wasm32")]
pub(crate) mod clipboard;
#[cfg(any(target_arch = "wasm32", test))]
mod connection_code;
mod constants;
pub(crate) mod entity_map;
mod messages;
mod plugin;
pub(crate) mod protocol;
pub(crate) mod resources;
pub(crate) mod session;
pub(crate) mod snapshot;
#[cfg(target_arch = "wasm32")]
pub(crate) mod webrtc;

pub use plugin::NetworkingPlugin;
