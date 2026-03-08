//! Networking plugin.
//!
//! Registers the networking resource for P2P communication.

use bevy::prelude::*;

use super::resources::NetworkConnection;

/// Plugin that manages P2P networking.
///
/// Initializes the network connection resource. Game-level networking
/// systems are registered by their respective plugins.
#[derive(Default)]
pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConnection>();
    }
}
