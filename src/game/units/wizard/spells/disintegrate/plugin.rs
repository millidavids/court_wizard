use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::state::InGameState;

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
                systems::handle_disintegrate_casting
                    .run_if(spell_is_primed(Spell::Disintegrate))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_beam_visuals,
                systems::apply_disintegrate_damage,
                systems::cleanup_beams_on_cancel,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
