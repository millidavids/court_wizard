use bevy::prelude::*;

use crate::state::AppState;

use super::components::DefendersActivated;
use super::systems;

/// Plugin that handles infantry units (both defenders and attackers).
///
/// Registers systems for:
/// - Initial spawn of defenders and attackers on game start
/// - Updating defender and attacker targeting
/// - Shared activation system for defenders
pub struct InfantryPlugin;

impl Plugin for InfantryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefendersActivated>().add_systems(
            OnEnter(AppState::InGame),
            (
                systems::spawn_initial_defenders,
                systems::spawn_initial_attackers,
            ),
        );
    }
}
