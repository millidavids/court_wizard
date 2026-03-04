use bevy::prelude::*;

/// Tracks the ogre's enrage state which activates at HP thresholds.
#[derive(Component)]
pub struct OgreEnrageState {
    /// Current enrage phase (0 = none, 1-3 = progressively enraged).
    pub phase: u8,
    /// Speed bonus from enrage (added to haste parameter).
    pub speed_bonus: f32,
    /// Damage bonus from enrage (added to DamageMultiplier).
    pub damage_bonus: f32,
}

impl OgreEnrageState {
    pub const fn new() -> Self {
        Self {
            phase: 0,
            speed_bonus: 0.0,
            damage_bonus: 0.0,
        }
    }
}

/// Cooldown timer for the ogre's melee attacks (separate from global attack cycle).
#[derive(Component)]
pub struct OgreAttackCooldown {
    pub time_remaining: f32,
}

impl OgreAttackCooldown {
    pub const fn new() -> Self {
        Self {
            time_remaining: 0.0,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.time_remaining -= delta;
    }

    pub fn is_ready(&self) -> bool {
        self.time_remaining <= 0.0
    }

    pub fn reset(&mut self, cooldown: f32) {
        self.time_remaining = cooldown;
    }
}

/// Re-export shared Knockback for backward compatibility.
pub use crate::game::units::components::Knockback as OgreKnockback;
