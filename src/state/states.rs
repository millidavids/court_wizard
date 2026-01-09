use bevy::prelude::*;

/// Game state machine representing the current screen/mode of the game.
///
/// States control which systems run and what entities are visible.
/// Transitions between states are handled by user input or game events.
///
/// # State Flow
///
/// - `StartMenu` → `GameRunning`: Player starts a new game
/// - `GameRunning` → `PauseMenu`: Player pauses the game
/// - `PauseMenu` → `GameRunning`: Player resumes the game
/// - `PauseMenu` → `StartMenu`: Player quits to main menu
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use the_game::GameState;
///
/// fn start_game(mut next_state: ResMut<NextState<GameState>>) {
///     next_state.set(GameState::GameRunning);
/// }
///
/// fn pause_game(mut next_state: ResMut<NextState<GameState>>) {
///     next_state.set(GameState::PauseMenu);
/// }
/// ```
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    /// Initial start menu where players begin.
    ///
    /// This is the default state when the game launches.
    /// Players can start a new game or configure settings.
    #[default]
    StartMenu,

    /// Active gameplay state.
    ///
    /// This is where the main game logic runs and players
    /// interact with the game world.
    GameRunning,

    /// Pause menu overlay during gameplay.
    ///
    /// Game logic is suspended while in this state.
    /// Players can resume, adjust settings, or return to the start menu.
    PauseMenu,
}
