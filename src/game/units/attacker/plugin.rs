use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::systems;

/// Plugin that handles enemy attacker units.
///
/// Registers systems for:
/// - Initial spawn of attackers on game start
/// - Updating attacker targeting toward nearest defender
pub struct AttackerPlugin;

impl Plugin for AttackerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::spawn_initial_attackers)
            .add_systems(
                Update,
                systems::update_attacker_targets.run_if(in_state(InGameState::Running)),
            );
    }
}
