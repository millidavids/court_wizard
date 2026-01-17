//! Pause menu main screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the pause menu main screen.
///
/// Used for cleanup when exiting the pause menu main state.
#[derive(Component)]
pub struct OnPauseMainScreen;

/// Stores the original colors for a button, used to compute hover/pressed states.
#[derive(Component)]
pub struct ButtonColors {
    /// The button's background color in its default state.
    pub background: Color,
    /// The button's border color in its default state.
    pub border: Color,
}

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
