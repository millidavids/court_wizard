use bevy::prelude::*;

/// Plugin for brew-specific systems.
/// Currently a placeholder for future per-brew plugins.
pub struct BrewsPlugin;

impl Plugin for BrewsPlugin {
    fn build(&self, _app: &mut App) {
        // Individual brew plugins will be registered here as needed
    }
}
