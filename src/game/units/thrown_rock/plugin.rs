use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::components::{RockProjectile, RockShadow, ThrownRock};
use super::messages::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;

pub struct ThrownRockPlugin;

impl Plugin for ThrownRockPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RockThrownMessage>()
            .add_systems(Startup, resources::preload_thrown_rock_assets)
            .add_systems(
                Update,
                (
                    spawn_rock_projectile.run_if(on_message::<RockThrownMessage>),
                    animate_rock_projectiles.run_if(any_with_component::<RockProjectile>),
                    tick_rock_lifetime.run_if(any_with_component::<ThrownRock>),
                    cleanup_sunk_rocks.run_if(any_with_component::<ThrownRock>),
                    units_attack_blocking_rocks.run_if(any_with_component::<ThrownRock>),
                    apply_spell_damage_to_rocks.run_if(any_with_component::<ThrownRock>),
                    destroy_dead_rocks
                        .after(PostCombatSet)
                        .after(apply_spell_damage_to_rocks)
                        .run_if(any_with_component::<ThrownRock>),
                    update_rock_damage_tint.run_if(any_with_component::<ThrownRock>),
                    cleanup_rock_shadows.run_if(any_with_component::<RockShadow>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
