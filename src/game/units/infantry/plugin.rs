use bevy::prelude::*;

use crate::game::run_conditions::{any_exist, coming_from_game_over};
use crate::game::units::MovementCalculationSet;
use crate::state::{AppState, InGameState};

use super::components::{DefendersActivated, Infantry};
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
                    systems::spawn_kings_guard,
                ),
            )
            .add_systems(
                OnEnter(InGameState::Running),
                (
                    systems::spawn_initial_defenders,
                    systems::spawn_initial_attackers,
                    systems::spawn_kings_guard,
                )
                    .run_if(coming_from_game_over),
            )
            .add_systems(
                Update,
                (
                    systems::check_defender_activation
                        .before(crate::game::plugin::VelocitySystemSet),
                    systems::update_infantry_targeting
                        .in_set(crate::game::plugin::VelocitySystemSet),
                    systems::infantry_movement.in_set(MovementCalculationSet),
                )
                    .run_if(any_exist::<Infantry>())
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
