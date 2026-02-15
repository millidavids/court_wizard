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
}
