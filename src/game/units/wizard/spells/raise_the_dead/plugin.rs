use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

/// Plugin for the Raise The Dead spell.
///
/// Manages the necromancy spell that resurrects corpses as hostile undead units.
pub struct RaiseTheDeadPlugin;

impl Plugin for RaiseTheDeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input) - runs on both host and guest
                systems::handle_raise_the_dead_casting
                    .run_if(spell_is_primed(Spell::RaiseTheDead))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
                // Guest wizard casting on host (network signals)
                systems::handle_raise_the_dead_casting_guest
                    .run_if(guest_spell_is_primed(Spell::RaiseTheDead))
                    .run_if(guest_input_or_wizard_casting)
                    .run_if(is_gameplay_running),
            ),
        );
    }
}
