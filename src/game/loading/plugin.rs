use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), systems::init_loading_progress)
            .add_systems(
                Update,
                systems::process_spawn_queue.run_if(in_state(AppState::Loading)),
            )
            .add_systems(OnExit(AppState::Loading), systems::cleanup_loading_progress);
    }
}
