use bevy::prelude::*;

use crate::game::plugin::{MovementSystemSet, VelocitySystemSet};
use crate::game::run_conditions::{any_exist, coming_from_game_over};
use crate::game::shared_systems::apply_separation;
use crate::state::InGameState;

use super::components::{King, KingSpawned};
use super::resources;
use super::systems;

pub struct KingPlugin;

impl Plugin for KingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KingSpawned>()
            .add_systems(Startup, resources::preload_king_assets)
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_king.run_if(coming_from_game_over),
            )
            .add_systems(
                Update,
                (
                    systems::update_king_targeting.in_set(VelocitySystemSet),
                    systems::king_movement.in_set(crate::game::units::MovementCalculationSet),
                    systems::king_cohesion_force
                        .after(apply_separation)
                        .before(MovementSystemSet),
                    systems::snap_kings_guard_to_king
                        .in_set(MovementSystemSet)
                        .after(systems::king_movement),
                )
                    .run_if(any_exist::<King>())
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
