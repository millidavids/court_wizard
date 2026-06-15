use bevy::prelude::*;

/// Marker for entities that should be despawned when exiting GameOver state.
#[derive(Component)]
pub(super) struct OnGameOverScreen;

/// Actions that can be triggered by buttons on the game over screen.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum GameOverButtonAction {
    /// Launch the next level immediately, skipping the wizard tower.
    NextLevel,
    PlayAgain,
    ReturnToTower,
    ReturnToMenu,
    /// End a roguelite run — saves run stats and returns to main menu.
    EndRun,
}
