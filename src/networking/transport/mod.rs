//! P2P transport layer using iroh (QUIC-based).
//!
//! Bridges iroh's async networking with Bevy's synchronous ECS via crossbeam channels.
//! The tokio runtime runs in a background thread; the bridge system syncs state each frame.

mod bridge;
mod codec;
mod connection;
mod runtime;

pub(crate) use bridge::TransportBridgePlugin;
pub(crate) use runtime::{TransportCommand, TransportHandle};
