use bevy::prelude::*;

use super::states::{
    AppState, InGameState, MenuState, MetaGameState, MultiplayerGameState, PauseMenuState,
    SplashState,
};
#[cfg(debug_assertions)]
use super::systems::{
    log_app_state_transitions, log_in_game_state_transitions, log_menu_state_transitions,
    log_meta_game_state_transitions, log_multiplayer_game_state_transitions,
    log_pause_menu_state_transitions, log_splash_state_transitions,
};

/// Manages all game states.
///
/// This plugin registers all state types and their transitions.
/// Individual game systems should use `NextState<T>` to trigger transitions.
///
/// # State Transitions
///
/// To change states from a system:
///
/// ```rust
/// use bevy::prelude::*;
/// use court_wizard::state::{AppState, MenuState};
///
/// fn start_game(mut next_state: ResMut<NextState<AppState>>) {
///     next_state.set(AppState::InGame);
/// }
///
/// fn open_settings(mut next_state: ResMut<NextState<MenuState>>) {
///     next_state.set(MenuState::Settings);
/// }
/// ```
///
/// # State-Dependent Systems
///
/// Use `.run_if(in_state(State))` to conditionally run systems:
///
/// ```rust
/// use bevy::prelude::*;
/// use court_wizard::state::AppState;
///
/// fn game_logic() {
///     // This system only runs when AppState::InGame is active
/// }
///
/// // In plugin build():
/// // app.add_systems(Update, game_logic.run_if(in_state(AppState::InGame)));
/// ```
///
/// # State-Based Setup/Cleanup
///
/// Use `OnEnter` and `OnExit` schedules for state-specific initialization:
///
/// ```rust
/// use bevy::prelude::*;
/// use court_wizard::state::AppState;
///
/// fn setup_game(mut commands: Commands) {
///     // Spawn game entities
/// }
///
/// fn cleanup_game(mut commands: Commands) {
///     // Despawn game entities
/// }
///
/// // In plugin build():
/// // app.add_systems(OnEnter(AppState::InGame), setup_game);
/// // app.add_systems(OnExit(AppState::InGame), cleanup_game);
/// ```
#[derive(Default)]
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        // Initialize primary state
        app.init_state::<AppState>();

        // Add sub-states
        app.add_sub_state::<MenuState>();
        app.add_sub_state::<InGameState>();
        app.add_sub_state::<PauseMenuState>();
        app.add_sub_state::<MetaGameState>();
        app.add_sub_state::<MultiplayerGameState>();
        app.add_sub_state::<SplashState>();

        // Optional: Add state transition logging for debugging
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (
                log_app_state_transitions,
                log_menu_state_transitions,
                log_in_game_state_transitions,
                log_pause_menu_state_transitions,
                log_meta_game_state_transitions,
                log_multiplayer_game_state_transitions,
                log_splash_state_transitions,
            ),
        );
    }
}
