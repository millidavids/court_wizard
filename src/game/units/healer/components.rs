use bevy::prelude::*;

use crate::game::units::components::Team;

/// Marker component for healer units.
#[derive(Component)]
pub struct Healer;

/// Homing heal bolt projectile fired by healers.
#[derive(Component)]
pub struct HealBolt {
    /// The entity this bolt is tracking.
    pub target: Entity,
    /// Movement speed of the bolt.
    pub speed: f32,
    /// The team that fired this bolt (to verify target validity).
    pub source_team: Team,
    /// Remaining lifetime before despawn.
    pub lifetime: f32,
}

/// Tracks time since last heal for cooldown.
#[derive(Component)]
pub struct HealerAttackTimer {
    /// Time since last heal (seconds).
    pub time_since_last_attack: f32,
}

impl HealerAttackTimer {
    pub const fn new() -> Self {
        Self {
            time_since_last_attack: 999.0, // Start high so can heal immediately
        }
    }
}
