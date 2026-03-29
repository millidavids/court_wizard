use bevy::prelude::*;

/// Primary application state.
///
/// Controls the top-level game flow. All game logic should be
/// conditioned on one of these states.
///
/// # State Transitions
///
/// - `Splash` → `MainMenu`: Splash screen animation completes
/// - `MainMenu` → `MetaGame`: Player selects a wizard and clicks Play
/// - `MetaGame` → `Loading`: Player clicks Start Next Battle
/// - `Loading` → `InGame`: Assets loaded and units spawned
/// - `InGame` → `MetaGame`: Player wins and clicks Continue (score screen)
/// - `InGame` → `Loading`: Player loses and clicks Try Again (immediate retry)
/// - `MetaGame` → `MainMenu`: Player clicks Return to Menu
/// - `InGame` → `MainMenu`: Player quits from pause or score screen
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[allow(dead_code)] // Variants will be used as game features are implemented
pub enum AppState {
    /// Splash screen - shows studio name before main menu.
    #[default]
    Splash,

    /// Main menu state - game is not running.
    MainMenu,

    /// Loading state - progressively spawning units to avoid blocking.
    Loading,

    /// Meta-game state - wizard tower progression between battles.
    MetaGame,

    /// Active gameplay state.
    InGame,

    /// Multiplayer loading state - progressively spawning multiplayer battlefield.
    MultiplayerLoading,

    /// Active multiplayer gameplay state.
    MultiplayerGame,
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

    /// Game mode selection screen (Story, Roguelite, Endless, Multiplayer).
    GameModeSelect,

    /// Roguelite modifier selection (game speed, enemy effectiveness, enemy count).
    RogueliteModifiers,

    /// Wizard type selection screen for picking or creating a wizard.
    WizardSelect,

    /// Compendium screen showing achievements, unlockables, and unit tracking.
    Compendium,

    /// Multiplayer lobby for P2P WebRTC connection setup.
    Multiplayer,
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
/// - `Running` → `ScoreScreen`: Game ends (win or lose)
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

    /// Score screen shown after battle ends (win or lose).
    ScoreScreen,

    /// Tutorial overlay — gameplay is frozen, tutorial panel is shown.
    Tutorial,
}

/// MetaGame sub-state for wizard tower progression screens.
///
/// This is a SubState that only exists when AppState::MetaGame is active.
/// When the meta-game is exited, this state is automatically cleaned up.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::MetaGame)]
pub enum MetaGameState {
    /// Wizard's Tower hub screen.
    #[default]
    WizardTower,

    /// Spell research screen with allocation sliders and commit.
    Study,

    /// Compendium screen accessible from wizard tower.
    Compendium,
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

    /// Compendium screen showing achievements, unlockables, and unit tracking.
    Compendium,
}

/// Splash screen sub-state.
///
/// This is a SubState that only exists when AppState::Splash is active.
/// Controls the sequence of splash screens shown before the main menu.
///
/// # State Transitions
///
/// - `Black` → `Language`: Brief black screen lets render pipeline initialize
/// - `Language` → `Engine`: Rust logo hold completes
/// - `Engine` → `Studio`: Bevy logo hold completes
/// - `Studio` → (AppState::MainMenu): Studio splash hold completes
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::Splash)]
pub enum SplashState {
    /// Brief black screen to let the render pipeline initialize.
    #[default]
    Black,

    /// Rust language logo splash.
    Language,

    /// Bevy engine logo splash.
    Engine,

    /// Studio branding splash.
    Studio,
}

/// Multiplayer game sub-state.
///
/// This is a SubState that only exists when AppState::MultiplayerGame is active.
/// Separate from InGameState to avoid any coupling with single-player systems.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::MultiplayerGame)]
pub enum MultiplayerGameState {
    /// Active multiplayer gameplay.
    #[default]
    Running,

    /// Escape menu overlay (gameplay continues in background).
    Paused,

    /// Spell book overlay (gameplay continues in background).
    SpellBook,

    /// Score screen shown after multiplayer battle ends.
    ScoreScreen,

    /// Connection lost — shows disconnect message with Return to Menu button.
    Disconnected,
}
