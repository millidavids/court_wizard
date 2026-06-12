//! Loading state cleanup.

use bevy::prelude::*;

use crate::game::loading::spawn_queue::SpawnQueue;

/// Cleans up loading resources when exiting loading state.
pub fn cleanup_loading_progress(mut commands: Commands) {
    commands.remove_resource::<SpawnQueue>();
}
