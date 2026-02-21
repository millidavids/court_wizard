use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{EntangleGroundEffect, EntangleIndicator};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct EntanglePlugin;

impl Plugin for EntanglePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_entangle_casting
                    .run_if(spell_is_primed(Spell::Entangle))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_entangle_indicator.run_if(any_exist::<EntangleIndicator>()),
                (
                    systems::fade_entangle_ground_effect,
                    systems::cleanup_entangle_ground_effect,
                )
                    .chain()
                    .run_if(any_exist::<EntangleGroundEffect>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
