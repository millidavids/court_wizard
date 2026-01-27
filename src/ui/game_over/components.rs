use bevy::prelude::*;

/// Marker for entities that should be despawned when exiting GameOver state.
#[derive(Component)]
pub struct OnGameOverScreen;

/// Actions that can be triggered by buttons on the game over screen.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum GameOverButtonAction {
    PlayAgain,
    ReturnToMenu,
}
