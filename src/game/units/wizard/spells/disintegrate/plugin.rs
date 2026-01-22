use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles disintegrate spell casting and behavior.
///
/// Registers systems for:
/// - Casting disintegrate beam with right-click
/// - Beam damage application
/// - Beam visual updates
/// - Cleanup when casting stops
pub struct DisintegratePlugin;

impl Plugin for DisintegratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_disintegrate_casting,
                systems::update_beam_visuals,
                systems::apply_disintegrate_damage,
                systems::cleanup_beams_on_cancel,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
