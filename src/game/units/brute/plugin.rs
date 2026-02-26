use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::components::*;
use super::messages::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::{PostCombatSet, VelocitySystemSet};

use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct BrutePlugin;

impl Plugin for BrutePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register message
            .add_message::<BruteAttackMessage>()
            .add_systems(Startup, resources::preload_brute_assets)
            // Update systems during Running state
            .add_systems(
                Update,
                (
                    update_brute_targeting.in_set(VelocitySystemSet),
                    track_brute_attack_target.in_set(VelocitySystemSet),
                    brute_movement.in_set(MovementCalculationSet),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<Brute>),
            )
            // AOE damage runs after combat when messages are present
            .add_systems(
                Update,
                brute_aoe_splash_damage
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(on_message::<BruteAttackMessage>),
            );
    }
}
