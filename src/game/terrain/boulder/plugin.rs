use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::components::{Boulder, BoulderHeat, BoulderProjectile, BoulderShadow};
use super::messages::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;

pub struct BoulderPlugin;

impl Plugin for BoulderPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BoulderThrownMessage>()
            .add_systems(Startup, resources::preload_boulder_assets)
            .add_systems(
                Update,
                (
                    spawn_rock_projectile.run_if(on_message::<BoulderThrownMessage>),
                    animate_rock_projectiles.run_if(any_with_component::<BoulderProjectile>),
                    tick_rock_lifetime.run_if(any_with_component::<Boulder>),
                    cleanup_sunk_rocks.run_if(any_with_component::<Boulder>),
                    units_attack_blocking_rocks.run_if(any_with_component::<Boulder>),
                    apply_spell_damage_to_rocks.run_if(any_with_component::<Boulder>),
                    destroy_dead_rocks
                        .after(PostCombatSet)
                        .after(apply_spell_damage_to_rocks)
                        .run_if(any_with_component::<Boulder>),
                    update_rock_damage_tint.run_if(any_with_component::<Boulder>),
                    cleanup_rock_shadows.run_if(any_with_component::<BoulderShadow>),
                    sync_teleported_rocks.run_if(any_with_component::<Boulder>),
                    tick_boulder_heat.run_if(any_with_component::<BoulderHeat>),
                )
                    .run_if(is_gameplay_running),
            );
    }
}
