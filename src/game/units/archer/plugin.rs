use bevy::prelude::*;

use super::systems::*;
use crate::game::run_conditions;
use crate::state::{AppState, InGameState};

pub struct ArcherPlugin;

impl Plugin for ArcherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (
                spawn_initial_defender_archers,
                spawn_initial_attacker_archers,
            ),
        )
        .add_systems(
            OnEnter(InGameState::Running),
            (
                spawn_initial_defender_archers,
                spawn_initial_attacker_archers,
            )
                .run_if(run_conditions::coming_from_game_over),
        )
        .add_systems(
            Update,
            update_archer_targeting.in_set(crate::game::plugin::VelocitySystemSet),
        )
        .add_systems(
            Update,
            archer_movement.in_set(crate::game::plugin::MovementSystemSet),
        )
        .add_systems(
            Update,
            (
                update_archer_movement_timers,
                archer_melee_combat,
                archer_ranged_combat,
                move_arrows,
                check_arrow_collisions,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
