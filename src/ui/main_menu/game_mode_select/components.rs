use bevy::prelude::*;

/// Marker component for entities that belong to the game mode select screen.
#[derive(Component)]
pub(super) struct OnGameModeSelectScreen;

/// Actions for game mode selection buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GameModeButtonAction {
    Story,
    Roguelite,
    Endless,
    Multiplayer,
    Back,
}

/// Marks a button as disabled (coming soon).
#[derive(Component)]
pub(super) struct DisabledModeButton;
