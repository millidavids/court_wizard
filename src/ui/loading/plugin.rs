use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

pub struct LoadingUiPlugin;

impl Plugin for LoadingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), systems::spawn_loading_screen)
            .add_systems(OnExit(AppState::Loading), systems::despawn_loading_screen)
            // Multiplayer loading reuses the same loading screen
            .add_systems(
                OnEnter(AppState::MultiplayerLoading),
                systems::spawn_loading_screen,
            )
            .add_systems(
                OnExit(AppState::MultiplayerLoading),
                systems::despawn_loading_screen,
            );
    }
}
