use bevy::prelude::*;

use crate::state::AppState;

use super::init;
use super::queue;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), init::init_loading_progress)
            .add_systems(
                Update,
                queue::process_spawn_queue.run_if(in_state(AppState::Loading)),
            )
            .add_systems(OnExit(AppState::Loading), queue::cleanup_loading_progress);
    }
}
