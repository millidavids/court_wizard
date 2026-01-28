use bevy::prelude::*;

use crate::game::units::components::Team;

/// Marker component for archer units.
#[derive(Component)]
pub struct Archer;

/// Attack range component for ranged units.
///
/// Defines the engagement distance for archers.
#[derive(Component, Clone, Copy)]
pub struct AttackRange {
    /// Optimal minimum range (archers try to stay at least this far)
    pub min_range: f32,
    /// Maximum range at which unit can attack
    pub max_range: f32,
}

/// Component marking an arrow projectile.
#[derive(Component)]
pub struct Arrow {
    /// Current velocity vector (includes gravity effects)
    pub velocity: Vec3,
    /// Damage dealt on impact
    pub damage: f32,
    /// The team that fired this arrow (to avoid friendly fire)
    pub source_team: Team,
}

/// Tracks time since archer stopped moving (for attack delay).
#[derive(Component)]
pub struct ArcherMovementTimer {
    /// Time elapsed since archer stopped moving
    pub time_since_stopped: f32,
    /// Whether the archer was moving in the previous frame
    pub was_moving: bool,
    /// Time since last ranged attack (for cooldown)
    pub time_since_last_attack: f32,
}

impl ArcherMovementTimer {
    pub const fn new() -> Self {
        Self {
            time_since_stopped: 0.0,
            was_moving: false,
            time_since_last_attack: 999.0, // Start high so can attack immediately
        }
    }

    /// Returns true if enough time has passed since stopping to allow attacking.
    pub fn can_attack(&self, required_delay: f32) -> bool {
        self.time_since_stopped >= required_delay
    }
}
