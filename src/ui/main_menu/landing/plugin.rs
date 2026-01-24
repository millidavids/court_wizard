//! Landing screen plugin.

use bevy::prelude::*;

use crate::state::MenuState;

use super::systems::{button_action, cleanup, keyboard_input, setup};

/// Plugin that manages the landing screen UI.
///
/// Registers systems for:
/// - Landing screen setup and cleanup
/// - Button interactions and visual feedback
/// - Menu navigation and state transitions
#[derive(Default)]
pub struct LandingPlugin;

impl Plugin for LandingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Landing), setup)
            .add_systems(OnExit(MenuState::Landing), cleanup)
            .add_systems(
                Update,
                (button_action, keyboard_input)
                    .run_if(in_state(MenuState::Landing)),
            );
    }
}
