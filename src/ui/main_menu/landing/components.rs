//! Landing screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the landing screen.
///
/// Used for cleanup when exiting the landing state.
#[derive(Component)]
pub(super) struct OnLandingScreen;

/// Actions that can be triggered by menu buttons.
///
/// Each variant corresponds to a specific action taken when
/// a button is pressed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuButtonAction {
    /// Go to the wizard select screen to pick or create a wizard.
    Play,

    /// Open the settings menu, transitioning to `MenuState::Settings`.
    Settings,

    /// Open the changelog screen, transitioning to `MenuState::Changelog`.
    Changelog,

    /// Open the instructions screen, transitioning to `MenuState::Instructions`.
    Instructions,

    /// Open the credits screen, transitioning to `MenuState::Credits`.
    Credits,

    /// Open the compendium screen, transitioning to `MenuState::Compendium`.
    Compendium,

    /// Open the multiplayer lobby, transitioning to `MenuState::Multiplayer`.
    Multiplayer,
}
