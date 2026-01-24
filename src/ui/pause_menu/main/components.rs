//! Pause menu main screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the pause menu main screen.
///
/// Used for cleanup when exiting the pause menu main state.
#[derive(Component)]
pub struct OnPauseMainScreen;

/// Actions that can be triggered by pause menu buttons.
///
/// Each variant corresponds to a specific action taken when
/// a button is pressed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuButtonAction {
    /// Continue the game, transitioning to `InGameState::Running`.
    Continue,

    /// Open the settings menu, transitioning to `PauseMenuState::Settings`.
    Settings,

    /// Exit to main menu, transitioning to `AppState::MainMenu`.
    Exit,
}
