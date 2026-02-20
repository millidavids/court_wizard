//! Multiplayer game plugin.
//!
//! Registers all multiplayer gameplay systems. This is completely independent
//! from `GamePlugin` — it reuses shared helper functions but has its own
//! system registrations under `AppState::MultiplayerGame`.

use bevy::prelude::*;

use crate::state::AppState;

use super::loading;

/// Plugin that manages multiplayer gameplay.
pub struct MultiplayerGamePlugin;

impl Plugin for MultiplayerGamePlugin {
    fn build(&self, app: &mut App) {
        // Multiplayer loading
        app.add_systems(
            OnEnter(AppState::MultiplayerLoading),
            loading::init_mp_loading,
        )
        .add_systems(
            Update,
            loading::process_mp_spawn_queue.run_if(in_state(AppState::MultiplayerLoading)),
        )
        .add_systems(
            OnExit(AppState::MultiplayerLoading),
            loading::cleanup_mp_loading,
        );

        // Camera setup on entering multiplayer game
        app.add_systems(
            OnEnter(AppState::MultiplayerGame),
            loading::setup_mp_camera,
        );

        // Camera restore on exiting multiplayer game
        app.add_systems(
            OnExit(AppState::MultiplayerGame),
            loading::restore_camera,
        );

        // TODO: Milestone 4 — host simulation + guest rendering systems
    }
}
