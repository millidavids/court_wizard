//! Pathfinding events for obstacle changes.

use bevy::prelude::*;

/// Event fired when an obstacle appears, changes, or is removed.
#[derive(Message)]
pub struct ObstacleChanged {
    /// World-space bounds of the obstacle.
    pub bounds: Rect,
    /// Type of obstacle change.
    pub obstacle_type: ObstacleType,
}

/// Types of obstacle changes.
#[derive(Clone, Copy, Debug)]
pub enum ObstacleType {
    /// Cell is completely blocked (walls, boulders).
    Blocked,
    /// Cell has increased movement cost (mud, water).
    /// The f32 value is the cost multiplier (e.g., 3.0 = 3x slower).
    #[allow(dead_code)]
    SlowTerrain(f32),
    /// Cell is a hazard (fire, poison) with very high movement cost (50x slower).
    /// Units will strongly avoid these areas unless no other path exists.
    Hazard,
    /// Obstacle was removed, reset to normal terrain.
    Removed,
}
