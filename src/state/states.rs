use bevy::prelude::*;

/// Primary application state.
///
/// Controls the top-level game flow. All game logic should be
/// conditioned on one of these states.
///
/// # State Transitions
///
/// - `MainMenu` → `InGame`: Player starts a new game
/// - `InGame` → `Paused`: Player pauses the game
/// - `Paused` → `InGame`: Player resumes the game
/// - `InGame` → `GameOver`: Player loses all lives or completes the game
/// - `GameOver` → `MainMenu`: Player returns to main menu
/// - `Paused` → `MainMenu`: Player quits to main menu
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[allow(dead_code)] // Variants will be used as game features are implemented
pub enum AppState {
    /// Main menu state - game is not running.
    #[default]
    MainMenu,

    /// Active gameplay state.
    InGame,

    /// Game is paused.
    Paused,

    /// Game over screen (win or lose).
    GameOver,
}

/// Menu navigation state.
///
/// This is a SubState that only exists when AppState::MainMenu is active.
/// When the main menu is exited, this state is automatically cleaned up.
///
/// # Automatic Cleanup
///
/// When AppState changes from MainMenu to any other state, MenuState is
/// automatically removed. When returning to MainMenu, MenuState starts at
/// its default (Landing).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::MainMenu)]
#[allow(dead_code)] // Variants will be used as menu screens are implemented
pub enum MenuState {
    /// Landing screen with Start Game and Settings buttons.
    #[default]
    Landing,

    /// Settings submenu.
    Settings,

    /// Credits screen.
    Credits,
}
