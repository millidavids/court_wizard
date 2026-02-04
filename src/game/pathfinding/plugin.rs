//! Pathfinding plugin.

use bevy::prelude::*;

use super::events::ObstacleChanged;
use super::systems::*;

/// Plugin that handles flow field pathfinding for units.
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register message channels
            .add_message::<ObstacleChanged>()
            // Initialize pathfinding grid at startup
            .add_systems(Startup, initialize_pathfinding)
            // Handle obstacle changes and async rebuilding in Update
            .add_systems(
                Update,
                (
                    update_king_position,
                    handle_obstacle_events,
                    spawn_rebuild_tasks,
                    apply_completed_rebuilds,
                )
                    .chain(),
            );
    }
}
