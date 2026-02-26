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

/// Knockback effect that moves a unit outward over time with decay.
/// Applied by ogre melee attacks. Decays linearly
/// for a "tumbling through dirt" feel.
#[derive(Component)]
pub struct OgreKnockback {
    /// Direction of knockback (normalized XZ).
    pub direction_x: f32,
    pub direction_z: f32,
    /// Initial knockback speed (units/s at full strength).
    pub speed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Time remaining before the effect expires.
    pub remaining: f32,
}

impl OgreKnockback {
    pub fn new(direction: Vec3, speed: f32, duration: f32) -> Self {
        let normalized = direction.normalize_or_zero();
        Self {
            direction_x: normalized.x,
            direction_z: normalized.z,
            speed,
            duration,
            remaining: duration,
        }
    }
}
