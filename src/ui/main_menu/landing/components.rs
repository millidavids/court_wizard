//! Landing screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the landing screen.
///
/// Used for cleanup when exiting the landing state.
#[derive(Component)]
pub struct OnLandingScreen;

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
