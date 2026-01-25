//! Pause menu main screen plugin.

use bevy::prelude::*;

use crate::state::PauseMenuState;

use super::systems::{button_action, cleanup, keyboard_input, setup};

/// Plugin that manages the pause menu main screen UI.
///
/// Registers systems for:
/// - Pause menu main screen setup and cleanup
/// - Button interactions and visual feedback
/// - Menu navigation and state transitions
#[derive(Default)]
pub struct PauseMainPlugin;

impl Plugin for PauseMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Main), setup)
            .add_systems(OnExit(PauseMenuState::Main), cleanup)
            .add_systems(
                Update,
                (button_action, keyboard_input).run_if(in_state(PauseMenuState::Main)),
            );
    }
}
