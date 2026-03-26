use bevy::prelude::*;

/// Marker for entities that should be despawned when exiting GameOver state.
#[derive(Component)]
pub(super) struct OnGameOverScreen;

/// Actions that can be triggered by buttons on the game over screen.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum GameOverButtonAction {
    PlayAgain,
    ReturnToTower,
    ReturnToMenu,
    /// End a roguelite run — saves run stats and returns to main menu.
    EndRun,
}
