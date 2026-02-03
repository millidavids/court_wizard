use bevy::prelude::*;

use super::components::{Archer, Arrow};
use super::systems::*;
use crate::game::run_conditions::{any_exist, coming_from_game_over};
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
                .run_if(coming_from_game_over),
        )
        .add_systems(
            Update,
            (
                update_archer_targeting.in_set(crate::game::plugin::VelocitySystemSet),
                archer_movement.in_set(crate::game::units::MovementCalculationSet),
                (
                    update_archer_movement_timers,
                    archer_melee_combat,
                    archer_ranged_combat,
                )
                    .chain(),
            )
                .run_if(any_exist::<Archer>())
                .run_if(in_state(InGameState::Running)),
        )
        .add_systems(
            Update,
            (move_arrows, check_arrow_collisions)
                .chain()
                .run_if(any_exist::<Arrow>())
                .run_if(in_state(InGameState::Running)),
        );
    }
}
