//! Pathfinding components.

use bevy::prelude::*;

/// Component that determines which flow field a unit should follow.
#[derive(Component, Clone)]
pub enum FlowFieldInfluence {
    /// Attackers flow toward the King.
    Attacker,
    /// Defenders flow toward King's target when activated, or rally to spawn point when not.
    Defender { spawn_pos: Vec2 },
}

/// Tracks whether a unit is stuck and triggers recovery nudges.
///
/// Auto-inserted on entities with `FlowFieldVelocity`. Every 30 frames,
/// checks if the unit has moved. After 3 consecutive stuck checks (~1.5s),
/// applies a perpendicular nudge force to break free.
#[derive(Component)]
pub struct StuckDetection {
    /// Position at last check.
    pub last_check_pos: Vec3,
    /// Frames since last stuck check.
    pub frames_since_check: u32,
    /// How many consecutive checks found the unit stuck.
    pub consecutive_stuck: u32,
}

impl Default for StuckDetection {
    fn default() -> Self {
        Self {
            last_check_pos: Vec3::ZERO,
            frames_since_check: 0,
            consecutive_stuck: 0,
        }
    }
}

/// Flow field velocity calculated from sampling the flow field.
///
/// This is combined with targeting and flocking velocities in unit movement systems.
#[derive(Component, Default)]
pub struct FlowFieldVelocity {
    pub velocity: Vec3,
    /// Pathfinding distance to goal (integration field cost).
    /// Use this instead of straight-line distance for weighting decisions.
    pub pathfinding_distance: f32,
    /// Terrain cost of the unit's current cell (1.0 = normal, high = hazard).
    /// Used by movement weighting to keep flow field influence high near hazards.
    pub terrain_cost: f32,
    /// True when the unit has reached its flow field objective (within satisfaction
    /// radius of rally point). Units at their destination with no targeting should
    /// stop moving entirely so flocking doesn't push them around.
    pub at_destination: bool,
}
