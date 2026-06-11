use bevy::prelude::*;

use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleType};
use crate::game::pathfinding::resources::PathfindingGrid;

/// Handles obstacle change events — updates base_costs for walls, hazards, and terrain.
///
/// With continuous rebuilding, no explicit rebuild trigger is needed; the next rebuild
/// cycle will automatically pick up the updated base_costs.
pub fn handle_obstacle_events(
    mut obstacle_events: MessageReader<ObstacleChanged>,
    mut pathfinding: ResMut<PathfindingGrid>,
) {
    for event in obstacle_events.read() {
        let affected_cells = if let Some(shape) = &event.shape {
            pathfinding.shape_filtered_cells(event.bounds, shape)
        } else {
            pathfinding.world_bounds_to_cells(event.bounds)
        };

        match event.obstacle_type {
            ObstacleType::Blocked => {
                pathfinding.mark_blocked(&affected_cells);
            }
            ObstacleType::SlowTerrain(multiplier) => {
                pathfinding.set_terrain_cost(&affected_cells, multiplier);
            }
            ObstacleType::Hazard(cost) => {
                pathfinding.set_terrain_cost(&affected_cells, cost);
            }
            ObstacleType::Removed => {
                pathfinding.set_terrain_cost(&affected_cells, 1.0);
            }
        }
    }
}
