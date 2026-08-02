use bevy::prelude::*;

use crate::game::time_travel::{TimeTravelState, load_time_travel_terrain};
use crate::state::AppState;

use super::init;
use super::queue;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Loading),
            // Swaps in the replayed level's terrain before the spawn queue is
            // built from those fields.
            (
                load_time_travel_terrain.run_if(resource_exists::<TimeTravelState>),
                init::init_loading_progress,
            )
                .chain(),
        )
        .add_systems(
            Update,
            queue::process_spawn_queue.run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnExit(AppState::Loading), queue::cleanup_loading_progress);
    }
}
