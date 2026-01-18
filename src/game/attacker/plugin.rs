use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles enemy attacker units.
///
/// Registers systems for:
/// - Spawning attackers periodically
/// - Updating attacker targeting toward the wizard
pub struct AttackerPlugin;

impl Plugin for AttackerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (systems::spawn_attackers, systems::update_attacker_targets)
                .run_if(in_state(InGameState::Running)),
        );
    }
}
