use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

pub struct LoadingUiPlugin;

impl Plugin for LoadingUiPlugin {
    fn build(&self, app: &mut App) {
        // Single-player loading completes in a few frames (no screen needed).
        // Multiplayer loading still shows a loading screen while syncing.
        app.add_systems(
            OnEnter(AppState::MultiplayerLoading),
            systems::spawn_loading_screen,
        )
        .add_systems(
            OnExit(AppState::MultiplayerLoading),
            systems::despawn_loading_screen,
        );
    }
}
