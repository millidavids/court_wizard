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
            // Update systems run in order
            .add_systems(
                Update,
                (
                    // Generate initial fields when King spawns
                    generate_initial_fields,
                    // Track King movement for attacker field
                    update_king_position,
                    // Update King's target for defender field
                    update_king_target,
                    // Handle obstacle changes
                    handle_obstacle_events,
                    // Apply completed async rebuilds
                    apply_completed_rebuilds,
                    // Sample flow fields (run before movement systems)
                    sample_flow_fields,
                )
                    .chain(),
            );
    }
}
