use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{WallOfFireEffect, WallOfFireSfx};
use super::systems;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};

pub struct WallOfFirePlugin;

impl Plugin for WallOfFirePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_wall_of_fire_cancel.run_if(spell_is_primed(Spell::WallOfFire)),
                // Local wizard casting (mouse input)
                systems::handle_wall_of_fire_casting
                    .run_if(spell_is_primed(Spell::WallOfFire))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::apply_wall_of_fire_damage.run_if(any_exist::<WallOfFireEffect>()),
                systems::fade_wall_of_fire.run_if(any_exist::<WallOfFireEffect>()),
                systems::spawn_wall_of_fire_smoke.run_if(any_exist::<WallOfFireEffect>()),
                systems::cleanup_wall_of_fire.run_if(any_exist::<WallOfFireEffect>()),
                systems::cleanup_wall_of_fire_sfx.run_if(any_exist::<WallOfFireSfx>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
