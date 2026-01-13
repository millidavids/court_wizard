use bevy::prelude::*;

use super::states::{AppState, MenuState};

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
/// use the_game::state::{AppState, MenuState};
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
/// use the_game::state::AppState;
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
/// use the_game::state::AppState;
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

        // Optional: Add state transition logging for debugging
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (log_app_state_transitions, log_menu_state_transitions),
        );
    }
}

/// Logs AppState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
fn log_app_state_transitions(app_state: Res<State<AppState>>) {
    if app_state.is_changed() {
        info!("AppState changed to: {:?}", app_state.get());
    }
}

/// Logs MenuState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
fn log_menu_state_transitions(menu_state: Option<Res<State<MenuState>>>) {
    if let Some(state) = menu_state {
        if state.is_changed() {
            info!("MenuState changed to: {:?}", state.get());
        }
    }
}
