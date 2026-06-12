use bevy::prelude::*;

use super::states::{
    AppState, InGameState, MenuState, MetaGameState, MultiplayerGameState, PauseMenuState,
    SplashState,
};

/// Logs AppState transitions for debugging.
///
/// Only enabled in debug builds.
///
/// Note: `AppState` is always present (it is the root state, initialised in
/// `StatePlugin::build`), so `Res<State<AppState>>` is used here instead of
/// `Option<Res<...>>`. All sub-states (`MenuState`, `InGameState`, etc.) are
/// only present when their parent state is active, which is why those logging
/// functions use `Option<Res<...>>`.
#[cfg(debug_assertions)]
pub(super) fn log_app_state_transitions(app_state: Res<State<AppState>>) {
    if app_state.is_changed() {
        info!("AppState changed to: {:?}", app_state.get());
    }
}

/// Logs MenuState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
pub(super) fn log_menu_state_transitions(menu_state: Option<Res<State<MenuState>>>) {
    if let Some(state) = menu_state
        && state.is_changed()
    {
        info!("MenuState changed to: {:?}", state.get());
    }
}

/// Logs InGameState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
pub(super) fn log_in_game_state_transitions(in_game_state: Option<Res<State<InGameState>>>) {
    if let Some(state) = in_game_state
        && state.is_changed()
    {
        info!("InGameState changed to: {:?}", state.get());
    }
}

/// Logs PauseMenuState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
pub(super) fn log_pause_menu_state_transitions(
    pause_menu_state: Option<Res<State<PauseMenuState>>>,
) {
    if let Some(state) = pause_menu_state
        && state.is_changed()
    {
        info!("PauseMenuState changed to: {:?}", state.get());
    }
}

/// Logs MetaGameState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
pub(super) fn log_meta_game_state_transitions(meta_game_state: Option<Res<State<MetaGameState>>>) {
    if let Some(state) = meta_game_state
        && state.is_changed()
    {
        info!("MetaGameState changed to: {:?}", state.get());
    }
}

/// Logs MultiplayerGameState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
pub(super) fn log_multiplayer_game_state_transitions(
    mp_state: Option<Res<State<MultiplayerGameState>>>,
) {
    if let Some(state) = mp_state
        && state.is_changed()
    {
        info!("MultiplayerGameState changed to: {:?}", state.get());
    }
}

/// Logs SplashState transitions for debugging.
///
/// Only enabled in debug builds.
#[cfg(debug_assertions)]
pub(super) fn log_splash_state_transitions(splash_state: Option<Res<State<SplashState>>>) {
    if let Some(state) = splash_state
        && state.is_changed()
    {
        info!("SplashState changed to: {:?}", state.get());
    }
}
