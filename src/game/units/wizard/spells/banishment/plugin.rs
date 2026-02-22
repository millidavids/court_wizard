use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::units::components::BanishedModifier;
use crate::game::run_conditions::is_gameplay_running;

pub struct BanishmentPlugin;

impl Plugin for BanishmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_banishment_casting
                    .run_if(spell_is_primed(Spell::Banishment))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_banishment_casting_guest
                    .run_if(guest_spell_is_primed(Spell::Banishment))
                    .run_if(guest_input_or_wizard_casting),
                systems::tick_banished_units.run_if(any_exist::<BanishedModifier>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
