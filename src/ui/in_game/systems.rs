//! In-game systems for input handling.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::state::InGameState;

/// Handles keyboard input during active gameplay.
///
/// - Escape: Pause the game, transitioning to `InGameState::Paused`
///
/// # Arguments
///
/// * `keyboard` - Keyboard input resource
/// * `next_in_game_state` - Resource for transitioning the `InGameState`
pub fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Paused);
    }
}
