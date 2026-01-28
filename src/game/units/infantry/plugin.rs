use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::{AppState, InGameState};

use super::components::DefendersActivated;
use super::systems;

/// Plugin that handles infantry units (both defenders and attackers).
///
/// Registers systems for:
/// - Initial spawn of defenders and attackers on game start
/// - Re-spawn when entering Running state from GameOver (for replay)
/// - Updating defender and attacker targeting
/// - Shared activation system for defenders
pub struct InfantryPlugin;

impl Plugin for InfantryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefendersActivated>()
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    systems::spawn_initial_defenders,
                    systems::spawn_initial_attackers,
                ),
            )
            .add_systems(
                OnEnter(InGameState::Running),
                (
                    systems::spawn_initial_defenders,
                    systems::spawn_initial_attackers,
                )
                    .run_if(run_conditions::coming_from_game_over),
            )
            .add_systems(
                Update,
                systems::update_infantry_targeting.in_set(crate::game::plugin::VelocitySystemSet),
            )
            .add_systems(
                Update,
                systems::infantry_movement.in_set(crate::game::plugin::MovementSystemSet),
            );
    }
}
