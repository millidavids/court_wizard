use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::units::components::PolymorphedModifier;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

pub struct PolymorphPlugin;

impl Plugin for PolymorphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // Local wizard casting (mouse input)
            systems::handle_polymorph_casting
                .run_if(spell_is_primed(Spell::Polymorph))
                .run_if(spell_input_not_blocked)
                .run_if(mouse_left_not_consumed)
                .run_if(mouse_held_or_wizard_casting)
                .run_if(is_spell_effects_active),
        );
        app.add_systems(
            Update,
            systems::tick_polymorphed_units
                .run_if(any_exist::<PolymorphedModifier>())
                .run_if(is_gameplay_running),
        );
    }
}
