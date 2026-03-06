//! Pause menu main screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the pause menu main screen.
///
/// Used for cleanup when exiting the pause menu main state.
#[derive(Component)]
pub(super) struct OnPauseMainScreen;

/// Actions that can be triggered by pause menu buttons.
///
/// Each variant corresponds to a specific action taken when
/// a button is pressed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PauseMenuButtonAction {
    /// Continue the game, transitioning to `InGameState::Running`.
    Continue,

    /// Open the settings menu, transitioning to `PauseMenuState::Settings`.
    Settings,

    /// Open the instructions screen, transitioning to `PauseMenuState::Instructions`.
    Instructions,

    /// Open the compendium screen, transitioning to `PauseMenuState::Compendium`.
    Compendium,

    /// Exit to main menu, transitioning to `AppState::MainMenu`.
    Exit,
}
