//! Plugin for the Teleport spell.

use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles the Teleport spell.
///
/// Registers systems for:
/// - Two-phase casting (destination placement, then source placement)
/// - Circle animations (pulsing effects)
/// - Unit teleportation
pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_teleport_casting,
                systems::update_circle_animations,
            )
                .run_if(in_state(InGameState::Running)),
        );
    }
}
