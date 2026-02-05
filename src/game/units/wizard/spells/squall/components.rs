//! Squall spell components.

use bevy::prelude::*;

use crate::game::units::DamageType;

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
}

impl SquallStorm {
    /// Creates a new squall storm at the specified position.
    pub const fn new(position: Vec3, radius: f32, empowerment: f32) -> Self {
        Self {
            position,
            radius,
            time_alive: 0.0,
            time_since_spawn: 0.0,
            empowerment,
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

/// Circle indicator for the squall storm during casting.
///
/// Shows the area of effect that will be targeted by ice projectiles.
#[derive(Component)]
pub(super) struct SquallCircleIndicator {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
    /// Empowerment multiplier.
    #[allow(dead_code)]
    pub empowerment: f32,
}

impl SquallCircleIndicator {
    /// Creates a new circle indicator.
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }

    /// Returns the current scale factor for pulse animation.
    ///
    /// Pulsates between 0.95 and 1.05 during cast time.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0; // Hz
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

/// Ice projectile component - falls from the sky and explodes on impact.
#[derive(Component)]
pub(super) struct IceProjectile {
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
    pub empowerment: f32,
}

impl IceProjectile {
    /// Creates a new ice projectile.
    pub const fn new(
        velocity: Vec3,
        damage: f32,
        explosion_radius: f32,
        radius: f32,
        empowerment: f32,
    ) -> Self {
        Self {
            velocity,
            damage,
            damage_type: DamageType::Frost,
            explosion_radius,
            radius,
            empowerment,
        }
    }
}

/// Ice explosion component - visual and damage effect on impact.
#[derive(Component)]
pub(super) struct IceExplosion {
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
