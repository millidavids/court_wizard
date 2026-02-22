use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::run_conditions::is_gameplay_running;

pub struct MarkOfDeathPlugin;

impl Plugin for MarkOfDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_mark_of_death_casting
                    .run_if(spell_is_primed(Spell::MarkOfDeath))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_mark_of_death_casting_guest
                    .run_if(guest_spell_is_primed(Spell::MarkOfDeath))
                    .run_if(guest_input_or_wizard_casting),
            )
                .run_if(is_gameplay_running),
        );
    }
}
