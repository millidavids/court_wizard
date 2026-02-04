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
}
