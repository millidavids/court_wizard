//! Networking plugin.
//!
//! Registers the networking resource and P2P transport layer.

use bevy::prelude::*;

use super::resources::NetworkConnection;
use super::transport::TransportBridgePlugin;

/// Plugin that manages P2P networking.
///
/// Initializes the network connection resource and the iroh-based transport
/// layer. Game-level networking systems are registered by their respective plugins.
#[derive(Default)]
pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConnection>()
            .add_plugins(TransportBridgePlugin);
    }
}
