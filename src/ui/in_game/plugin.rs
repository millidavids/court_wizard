//! In-game UI plugin.

use bevy::prelude::*;

use crate::state::InGameState;

use super::systems::keyboard_input;

/// Plugin that manages in-game UI and input handling.
///
/// Registers systems for:
/// - Keyboard input during active gameplay (e.g., pause on Escape)
#[derive(Default)]
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            keyboard_input.run_if(in_state(InGameState::Running)),
        );
    }
}
