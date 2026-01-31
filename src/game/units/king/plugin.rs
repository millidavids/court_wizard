use bevy::prelude::*;

use crate::game::plugin::{MovementSystemSet, VelocitySystemSet};
use crate::game::run_conditions;
use crate::game::shared_systems::apply_separation;
use crate::state::{AppState, InGameState};

use super::components::KingSpawned;
use super::systems;

pub struct KingPlugin;

impl Plugin for KingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KingSpawned>()
            .add_systems(OnEnter(AppState::InGame), systems::spawn_king)
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_king.run_if(run_conditions::coming_from_game_over),
            )
            .add_systems(
                Update,
                systems::update_king_targeting.in_set(VelocitySystemSet),
            )
            .add_systems(Update, systems::king_movement.in_set(MovementSystemSet))
            .add_systems(
                Update,
                systems::king_cohesion_aura
                    .after(apply_separation)
                    .before(MovementSystemSet)
                    .run_if(in_state(InGameState::Running)),
            )
            .add_systems(
                Update,
                systems::snap_kings_guard_to_king
                    .in_set(MovementSystemSet)
                    .after(systems::king_movement),
            );
    }
}
