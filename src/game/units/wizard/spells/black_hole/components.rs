//! Black Hole spell components.

use bevy::prelude::*;

use super::constants::*;
use crate::game::units::DamageType;

/// Black hole spell component.
///
/// Creates a gravitational sphere that pulls units inward while making them spiral.
/// Deals damage to units in contact with the sphere, increasing over time.
#[derive(Component)]
pub(crate) struct BlackHole {
    /// Center position of the black hole in world space.
    pub position: Vec3,
    /// Current radius of the black hole sphere.
    pub current_radius: f32,
    /// Maximum radius the black hole can reach.
    pub max_radius: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time since the black hole was spawned (for growth animation).
    pub time_alive: f32,
    /// Time since last damage tick.
    pub time_since_damage: f32,
    /// Empowerment multiplier for spell effectiveness.
    pub empowerment: f32,
}

impl BlackHole {
    /// Creates a new black hole at the specified position.
    pub fn new(position: Vec3, max_radius: f32, empowerment: f32) -> Self {
        Self {
            position,
            current_radius: 0.0,
            max_radius,
            damage_type: DAMAGE_TYPE,
            time_alive: 0.0,
            time_since_damage: 0.0,
            empowerment,
        }
    }

    /// Returns the current radius based on growth time.
    pub fn calculate_current_radius(&mut self) -> f32 {
        let growth_factor = (self.time_alive / GROWTH_TIME).min(1.0);
        self.current_radius = self.max_radius * growth_factor;
        self.current_radius
    }

    /// Returns the gravitational pull strength at this moment.
    /// Increases linearly with time the black hole has existed.
    pub fn gravitational_strength(&self) -> f32 {
        let time_factor = (self.time_alive / GRAVITY_RAMP_TIME).min(1.0);
        let base_strength =
            BASE_GRAVITY_STRENGTH + (MAX_GRAVITY_STRENGTH - BASE_GRAVITY_STRENGTH) * time_factor;
        base_strength * self.empowerment
    }

    /// Returns damage per tick, scaled by empowerment.
    pub fn damage_per_tick(&self) -> f32 {
        BASE_DAMAGE_PER_TICK * self.empowerment
    }

    /// Returns true if a position is within the black hole sphere.
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.position.distance(point) <= self.current_radius
    }

    /// Checks if enough time has passed for another damage tick.
    pub fn should_damage(&self) -> bool {
        self.time_since_damage >= DAMAGE_INTERVAL
    }

    /// Resets the damage timer.
    pub fn reset_damage_timer(&mut self) {
        self.time_since_damage = 0.0;
    }

    /// Updates timers.
    pub fn update_timers(&mut self, delta: f32) {
        self.time_alive += delta;
        self.time_since_damage += delta;
        self.calculate_current_radius();
    }

    /// Returns true if the black hole has expired.
    pub fn is_expired(&self) -> bool {
        self.time_alive >= LIFETIME
    }
}

/// Component tracking how long a unit has been inside the black hole.
#[derive(Component)]
pub(super) struct UnitInBlackHole {
    /// Time the unit has been in contact with the black hole sphere (seconds).
    pub time_inside: f32,
}

impl UnitInBlackHole {
    pub fn new() -> Self {
        Self { time_inside: 0.0 }
    }

    /// Returns damage multiplier based on time inside (1.0 to MAX_DAMAGE_MULTIPLIER).
    pub fn damage_multiplier(&self) -> f32 {
        let ramp_factor = (self.time_inside / DAMAGE_RAMP_TIME).min(1.0);
        1.0 + (MAX_DAMAGE_MULTIPLIER - 1.0) * ramp_factor
    }
}
