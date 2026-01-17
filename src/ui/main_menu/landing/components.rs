//! Landing screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the landing screen.
///
/// Used for cleanup when exiting the landing state.
#[derive(Component)]
pub struct OnLandingScreen;

/// Stores the original colors for a button, used to compute hover/pressed states.
#[derive(Component)]
pub struct ButtonColors {
    /// The button's background color in its default state.
    pub background: Color,
    /// The button's border color in its default state.
    pub border: Color,
}

/// Actions that can be triggered by menu buttons.
///
/// Each variant corresponds to a specific action taken when
/// a button is pressed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuButtonAction {
    /// Start a new game, transitioning to `AppState::InGame`.
    StartGame,

    /// Open the settings menu, transitioning to `MenuState::Settings`.
    Settings,
}
