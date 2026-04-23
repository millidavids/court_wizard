//! Pause menu main screen plugin.

use bevy::prelude::*;

use crate::state::PauseMenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{escape_to_running, handle_scroll};

use super::components::ScrollablePauseStats;
use super::systems::{button_action, setup};

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
            .add_systems(
                OnExit(PauseMenuState::Main),
                crate::ui::systems::cleanup_screen::<super::components::OnPauseMainScreen>,
            )
            .add_systems(
                Update,
                button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(PauseMenuState::Main)),
            )
            .add_systems(
                Update,
                (escape_to_running, handle_scroll::<ScrollablePauseStats>)
                    .run_if(in_state(PauseMenuState::Main)),
            );
    }
}
