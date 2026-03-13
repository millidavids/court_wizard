use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct BerserkerRagePlugin;

impl Plugin for BerserkerRagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // Local wizard casting (mouse input)
            systems::handle_berserker_rage_casting
                .run_if(spell_is_primed(Spell::BerserkerRage))
                .run_if(spell_input_not_blocked)
                .run_if(mouse_left_not_consumed)
                .run_if(mouse_held_or_wizard_casting)
                .run_if(is_spell_effects_active),
        );
    }
}
