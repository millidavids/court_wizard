use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::HasteIndicator;
use super::systems;
use crate::game::run_conditions::is_gameplay_running;

pub struct HastePlugin;

impl Plugin for HastePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_haste_casting
                    .run_if(spell_is_primed(Spell::Haste))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_haste_casting_guest
                    .run_if(guest_spell_is_primed(Spell::Haste))
                    .run_if(guest_input_or_wizard_casting),
                systems::update_haste_indicator.run_if(any_exist::<HasteIndicator>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
