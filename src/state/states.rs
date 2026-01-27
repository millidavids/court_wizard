use bevy::prelude::*;

/// Primary application state.
///
/// Controls the top-level game flow. All game logic should be
/// conditioned on one of these states.
///
/// # State Transitions
///
/// - `MainMenu` → `InGame`: Player starts a new game
/// - `InGame` → `MainMenu`: Player quits to main menu from pause or game over
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[allow(dead_code)] // Variants will be used as game features are implemented
pub enum AppState {
    /// Main menu state - game is not running.
    #[default]
    MainMenu,

    /// Active gameplay state.
    InGame,
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

/// InGame sub-state.
///
/// This is a SubState that only exists when AppState::InGame is active.
/// When the InGame state is exited, this state is automatically cleaned up.
///
/// # State Transitions
///
/// - `Running` → `Paused`: Player presses Escape
/// - `Paused` → `Running`: Player selects Continue from pause menu
/// - `Running` → `SpellBook`: Player clicks Spells button
/// - `SpellBook` → `Running`: Player selects a spell or closes spell book
/// - `Running` → `GameOver`: Game ends (win or lose)
/// - `GameOver` → `Running`: Player clicks Play Again
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum InGameState {
    /// Active gameplay.
    #[default]
    Running,

    /// Game is paused.
    Paused,

    /// Spell selection screen.
    SpellBook,

    /// Game over screen (win or lose).
    GameOver,
}

/// Pause menu navigation state.
///
/// This is a SubState that only exists when InGameState::Paused is active.
/// When the pause state is exited, this state is automatically cleaned up.
///
/// # Automatic Cleanup
///
/// When InGameState changes from Paused to Running, PauseMenuState is
/// automatically removed. When returning to Paused, PauseMenuState starts at
/// its default (Main).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(InGameState = InGameState::Paused)]
pub enum PauseMenuState {
    /// Pause menu main screen with Continue, Settings, and Exit buttons.
    #[default]
    Main,

    /// Settings submenu (identical to main menu settings).
    Settings,
}
