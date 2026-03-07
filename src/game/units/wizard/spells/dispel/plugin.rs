use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{DispelCooldown, DispelImpact, DispelProjectile};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct DispelPlugin;

impl Plugin for DispelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::tick_dispel_cooldown
                    .run_if(any_exist::<DispelCooldown>())
                    .run_if(is_spell_effects_active),
                systems::handle_dispel_casting
                    .run_if(spell_is_primed(Spell::Dispel))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(is_spell_effects_active),
                systems::move_dispel_projectiles
                    .run_if(any_exist::<DispelProjectile>())
                    .run_if(is_spell_effects_active),
                systems::update_dispel_impacts
                    .run_if(any_exist::<DispelImpact>())
                    .run_if(is_spell_effects_active),
            ),
        );
    }
}
