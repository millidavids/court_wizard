use bevy::prelude::*;

/// Primary application state.
///
/// Controls the top-level game flow. All game logic should be
/// conditioned on one of these states.
///
/// # State Transitions
///
/// - `MainMenu` → `Loading`: Player starts a new game
/// - `Loading` → `InGame`: Assets loaded and units spawned
/// - `InGame` → `MainMenu`: Player quits to main menu from pause or game over
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[allow(dead_code)] // Variants will be used as game features are implemented
pub enum AppState {
    /// Main menu state - game is not running.
    #[default]
    MainMenu,

    /// Loading state - progressively spawning units to avoid blocking.
    Loading,

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

    /// Changelog screen.
    Changelog,

    /// Instructions screen explaining gameplay mechanics.
    Instructions,

    /// Credits screen.
    Credits,

    /// Wizard type selection screen for starting a new save.
    WizardSelect,

    /// Save file selection screen for continuing an existing save.
    SaveSelect,
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
/// - `Running` → `CauldronMenu`: Player clicks Cauldron button
/// - `CauldronMenu` → `Running`: Player selects a brew or closes cauldron menu
/// - `Running` → `GameOver`: Game ends (win or lose)
/// - `GameOver` → `WizardTower`: Player wins and clicks Continue
/// - `GameOver` → `Loading`: Player loses and clicks Try Again (immediate retry)
/// - `WizardTower` → `Loading`: Player clicks Start Next Battle
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

    /// Cauldron brew selection screen.
    CauldronMenu,

    /// Game over screen (win or lose).
    GameOver,

    /// Wizard's Tower screen - progression and maintenance between battles.
    WizardTower,
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

    /// Instructions screen explaining gameplay mechanics.
    Instructions,
}
