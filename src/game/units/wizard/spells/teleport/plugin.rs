//! Plugin for the Teleport spell.

use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{TeleportDestinationCircle, TeleportSourceCircle};
use super::systems;
use crate::game::run_conditions::is_gameplay_running;

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
                // Local wizard casting (mouse input)
                systems::handle_teleport_casting
                    .run_if(spell_is_primed(Spell::Teleport))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_right_not_held)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_teleport_casting_guest
                    .run_if(guest_spell_is_primed(Spell::Teleport))
                    .run_if(guest_input_or_wizard_casting),
                systems::update_circle_animations.run_if(
                    any_exist::<TeleportDestinationCircle>()
                        .or(any_exist::<TeleportSourceCircle>()),
                ),
            )
                .run_if(is_gameplay_running),
        );
    }
}
