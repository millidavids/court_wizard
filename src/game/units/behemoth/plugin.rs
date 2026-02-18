use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::components::*;
use super::messages::*;
use super::resources;
use super::systems::*;
use crate::game::plugin::VelocitySystemSet;

use crate::game::shared_systems;
use crate::game::units::MovementCalculationSet;
use crate::state::InGameState;

pub struct BehemothPlugin;

impl Plugin for BehemothPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register message
            .add_message::<BehemothAttackMessage>()
            .add_systems(Startup, resources::preload_behemoth_assets)
            // Update systems during Running state
            .add_systems(
                Update,
                (
                    update_behemoth_targeting.in_set(VelocitySystemSet),
                    track_behemoth_attack_target.in_set(VelocitySystemSet),
                    behemoth_movement.in_set(MovementCalculationSet),
                )
                    .run_if(in_state(InGameState::Running))
                    .run_if(any_with_component::<Behemoth>),
            )
            // AOE damage runs after combat when messages are present
            .add_systems(
                Update,
                behemoth_aoe_splash_damage
                    .after(shared_systems::combat)
                    .run_if(in_state(InGameState::Running))
                    .run_if(on_message::<BehemothAttackMessage>),
            );
    }
}
