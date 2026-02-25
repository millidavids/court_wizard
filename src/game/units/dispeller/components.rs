use bevy::prelude::*;

use crate::game::units::components::Team;

/// Marker component for dispeller units.
#[derive(Component)]
pub struct Dispeller;

/// Tracks channeling progress when a dispeller is removing a spell effect.
#[derive(Component)]
pub struct DispelChanneling {
    /// The spell effect entity being dispelled.
    pub target_entity: Entity,
    /// Time spent channeling so far (seconds).
    pub elapsed: f32,
}

/// Straight-line magic bolt projectile fired by dispellers.
#[derive(Component)]
pub struct DispellerBolt {
    /// Current velocity vector.
    pub velocity: Vec3,
    /// Damage dealt on impact.
    pub damage: f32,
    /// The team that fired this bolt (to avoid friendly fire).
    pub source_team: Team,
    /// Remaining lifetime before despawn.
    pub lifetime: f32,
}

/// Tracks time since last ranged attack for cooldown.
#[derive(Component)]
pub struct DispellerAttackTimer {
    /// Time since last ranged attack (seconds).
    pub time_since_last_attack: f32,
}

impl DispellerAttackTimer {
    pub const fn new() -> Self {
        Self {
            time_since_last_attack: 999.0, // Start high so can attack immediately
        }
    }
}
