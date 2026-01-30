//! Plugin for the Teleport spell.

use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::state::InGameState;

/// Plugin that handles the Teleport spell.
///
/// Registers systems for:
/// - Two-phase casting (destination placement, then source placement)
/// - Circle animations (pulsing effects)
/// - Unit teleportation
pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_teleport_cancel.run_if(spell_is_primed(Spell::Teleport)),
                systems::handle_teleport_casting
                    .run_if(spell_is_primed(Spell::Teleport))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_right_not_held)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_circle_animations,
            )
                .run_if(in_state(InGameState::Running)),
        );
    }
}
