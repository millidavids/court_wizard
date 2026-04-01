use std::collections::HashSet;

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

/// Reduces incoming melee (non-spell) damage by a multiplier.
/// Applied in the shared combat system.
#[derive(Component)]
pub struct MeleeDamageReduction {
    /// Fraction of damage taken (0.3 = takes 30%, blocks 70%).
    pub multiplier: f32,
}

/// Marker component for the charge direction indicator entity.
#[derive(Component)]
pub struct OgreChargeIndicator;

/// State machine for the ogre's charge ability.
#[derive(Component)]
pub enum OgreChargeState {
    /// Waiting for cooldown to expire before next charge.
    Idle { cooldown: f32 },
    /// Picking a target at moderate distance.
    Targeting,
    /// Growing the red fill inside an outline rectangle before charging.
    Telegraphing {
        elapsed: f32,
        direction: Vec3,
        target_distance: f32,
        indicators: ChargeIndicators,
    },
    /// Dashing forward at high speed, dealing damage to units in path.
    Charging {
        direction: Vec3,
        distance_traveled: f32,
        max_distance: f32,
        hit_entities: HashSet<Entity>,
    },
    /// Brief pause after charge completes.
    Recovery { timer: f32 },
}

pub struct ChargeIndicators {
    pub left: Entity,
    pub right: Entity,
    pub near: Entity,
    pub far: Entity,
    pub fill: Entity,
    pub fill_material: Handle<StandardMaterial>,
}

impl ChargeIndicators {
    pub fn all(&self) -> [Entity; 5] {
        [self.left, self.right, self.near, self.far, self.fill]
    }
}

impl OgreChargeState {
    pub fn is_movement_locked(&self) -> bool {
        matches!(
            self,
            Self::Telegraphing { .. } | Self::Charging { .. } | Self::Recovery { .. }
        )
    }
}
