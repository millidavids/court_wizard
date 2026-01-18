use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles friendly defender units.
///
/// Registers systems for:
/// - Spawning defenders periodically
/// - Updating defender targeting toward nearest attacker
/// - Combat between defenders and attackers
pub struct DefenderPlugin;

impl Plugin for DefenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::spawn_defenders,
                systems::update_defender_targets,
                systems::combat,
            )
                .run_if(in_state(InGameState::Running)),
        );
    }
}
