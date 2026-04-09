//! Squall spell components.

use bevy::prelude::*;

use crate::game::units::DamageType;

/// Talent parameters computed from active talent selections.
/// Stored on each SquallStorm entity so talent logic can reference it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SquallTalentParams {
    // Tier 1: numeric modifiers
    pub damage_mult: f32,
    pub radius_mult: f32,
    pub spawn_rate_mult: f32,
    // Tier 2: behavioral flags
    pub permafrost: bool,
    pub hailstones: bool,
    pub sleet_storm: bool,
    // Tier 3: transformative flags
    pub absolute_zero: bool,
    pub blizzard: bool,
    pub ice_age: bool,
}

impl Default for SquallTalentParams {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            radius_mult: 1.0,
            spawn_rate_mult: 1.0,
            permafrost: false,
            hailstones: false,
            sleet_storm: false,
            absolute_zero: false,
            blizzard: false,
            ice_age: false,
        }
    }
}

/// Squall storm component - invisible marker entity that spawns ice projectiles.
///
/// The storm persists as long as the wizard maintains concentration.
/// Periodically spawns ice projectiles that rain down within the storm radius.
#[derive(Component)]
pub(crate) struct SquallStorm {
    /// Center position of the storm in world space.
    pub position: Vec3,
    /// Radius of the storm area.
    pub radius: f32,
    /// Time since the storm was created (for animations/effects).
    pub time_alive: f32,
    /// Time since last ice projectile spawn.
    pub time_since_spawn: f32,
    /// Empowerment multiplier for spell effectiveness.
    pub empowerment: f32,
    /// Talent parameters for this storm instance.
    pub talent_params: SquallTalentParams,
}

impl SquallStorm {
    /// Creates a new squall storm at the specified position.
    pub fn new(
        position: Vec3,
        radius: f32,
        empowerment: f32,
        talent_params: SquallTalentParams,
    ) -> Self {
        Self {
            position,
            radius,
            time_alive: 0.0,
            time_since_spawn: 0.0,
            empowerment,
            talent_params,
        }
    }

    /// Updates timers.
    pub fn update_timers(&mut self, delta: f32) {
        self.time_alive += delta;
        self.time_since_spawn += delta;
    }

    /// Resets the spawn timer.
    pub fn reset_spawn_timer(&mut self) {
        self.time_since_spawn = 0.0;
    }
}

/// Ice projectile component - falls from the sky and explodes on impact.
#[derive(Component)]
pub(crate) struct IceProjectile {
    /// Current velocity of the projectile.
    pub velocity: Vec3,
    /// Damage dealt by the explosion.
    pub damage: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Radius of the explosion.
    pub explosion_radius: f32,
    /// Collision radius of the projectile itself.
    #[allow(dead_code)]
    pub radius: f32,
    /// Empowerment multiplier.
    #[allow(dead_code)]
    pub empowerment: f32,
    /// Whether this projectile is a hailstone (larger, more damage).
    pub is_hailstone: bool,
    /// Whether this projectile should leave frozen ground on impact (Ice Age talent).
    pub ice_age: bool,
}

impl IceProjectile {
    /// Creates a new ice projectile.
    pub fn new(
        velocity: Vec3,
        damage: f32,
        explosion_radius: f32,
        radius: f32,
        empowerment: f32,
        is_hailstone: bool,
        ice_age: bool,
    ) -> Self {
        Self {
            velocity,
            damage,
            damage_type: DamageType::Frost,
            explosion_radius,
            radius,
            empowerment,
            is_hailstone,
            ice_age,
        }
    }
}

/// Ice explosion component - visual and damage effect on impact.
#[derive(Component)]
pub(crate) struct IceExplosion {
    /// Center point of the explosion.
    pub origin: Vec3,
    /// Maximum radius the explosion will reach.
    pub max_radius: f32,
    /// Damage dealt to units in the explosion.
    pub damage: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time the explosion has been active (in seconds).
    pub time_alive: f32,
    /// Whether damage has been applied yet (one-time damage).
    pub damage_applied: bool,
    /// Empowerment multiplier.
    #[allow(dead_code)]
    pub empowerment: f32,
}

impl IceExplosion {
    /// Creates a new ice explosion.
    pub const fn new(origin: Vec3, max_radius: f32, damage: f32, empowerment: f32) -> Self {
        Self {
            origin,
            max_radius,
            damage,
            damage_type: DamageType::Frost,
            time_alive: 0.0,
            damage_applied: false,
            empowerment,
        }
    }

    /// Returns the current radius of the explosion based on how long it's been active.
    pub fn current_radius(&self, growth_time: f32) -> f32 {
        if growth_time <= 0.0 {
            return self.max_radius;
        }

        let growth_factor = (self.time_alive / growth_time).min(1.0);
        self.max_radius * growth_factor
    }
}

/// Frozen ground patch left by the Ice Age talent.
/// Slows enemies that walk over it.
#[derive(Component)]
pub(crate) struct FrozenGround {
    /// Center position of the frozen patch.
    pub position: Vec3,
    /// Radius of the frozen patch.
    pub radius: f32,
    /// Time remaining before this patch melts.
    pub time_remaining: f32,
}

impl FrozenGround {
    pub fn new(position: Vec3, radius: f32, duration: f32) -> Self {
        Self {
            position,
            radius,
            time_remaining: duration,
        }
    }
}

/// Annulus ring reticle that persists while the storm is active.
#[derive(Component)]
pub(crate) struct SquallStormRing {
    /// Time alive for pulse animation.
    pub time_alive: f32,
}

/// Swirling snow particle in the squall storm area.
#[derive(Component)]
pub(crate) struct SnowParticle {
    /// Current velocity for swirling movement.
    pub velocity: Vec3,
    /// Time this particle has been alive.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base visual size.
    pub base_size: f32,
    /// Phase offset for animation variation.
    pub phase: f32,
}

/// Stacking slow from Absolute Zero talent.
/// Tracks accumulated slow separately from `SlowMovementModifier` so it can stack.
/// Decays after the unit leaves the zone or channeling stops.
#[derive(Component)]
pub(crate) struct AbsoluteZeroSlow {
    /// Current accumulated slow modifier (negative, e.g., -0.3 = 30% slow).
    pub accumulated_slow: f32,
    /// Time remaining before the slow decays (resets while in zone).
    pub decay_timer: f32,
}
