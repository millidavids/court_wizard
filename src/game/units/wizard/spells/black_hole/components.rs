//! Black Hole spell components.

use bevy::prelude::*;

use super::constants::*;
use crate::game::units::DamageType;

/// Pre-computed talent parameters for a black hole instance.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BlackHoleTalentParams {
    /// Gravity strength multiplier (T1-0 Denser Core).
    pub gravity_mult: f32,
    /// Max radius multiplier (T1-1 Expansive Void).
    pub radius_mult: f32,
    /// Damage multiplier (T1-1 Expansive Void).
    pub damage_mult: f32,
    /// T2-0 Event Horizon: units in inner zone take double damage.
    pub event_horizon: bool,
    /// T2-1 Crushing Pressure: slow units inside.
    pub crushing_pressure: bool,
    /// T2-2 Void Siphon: heal defenders with damage dealt.
    pub void_siphon: bool,
    /// T3-0 Singularity: collapse damage on expiration.
    pub singularity: bool,
    /// T3-2 Dimensional Rift: periodic teleport + burst.
    pub dimensional_rift: bool,
}

impl Default for BlackHoleTalentParams {
    fn default() -> Self {
        Self {
            gravity_mult: 1.0,
            radius_mult: 1.0,
            damage_mult: 1.0,
            event_horizon: false,
            crushing_pressure: false,
            void_siphon: false,
            singularity: false,
            dimensional_rift: false,
        }
    }
}

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
    /// Talent parameters for this instance.
    pub talent_params: BlackHoleTalentParams,
    /// Time since last Dimensional Rift pulse (seconds).
    pub time_since_rift_pulse: f32,
}

impl BlackHole {
    /// Creates a new black hole at the specified position.
    pub fn new(
        position: Vec3,
        max_radius: f32,
        empowerment: f32,
        talent_params: BlackHoleTalentParams,
    ) -> Self {
        Self {
            position,
            current_radius: 0.0,
            max_radius,
            damage_type: DAMAGE_TYPE,
            time_alive: 0.0,
            time_since_damage: 0.0,
            empowerment,
            talent_params,
            time_since_rift_pulse: 0.0,
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
        base_strength * self.empowerment * self.talent_params.gravity_mult
    }

    /// Returns damage per tick, scaled by empowerment and talent damage multiplier.
    pub fn damage_per_tick(&self) -> f32 {
        BASE_DAMAGE_PER_TICK * self.empowerment * self.talent_params.damage_mult
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
        self.time_since_rift_pulse += delta;
        self.calculate_current_radius();
    }

    /// Returns true if the black hole has expired.
    pub fn is_expired(&self) -> bool {
        self.time_alive >= LIFETIME
    }
}

/// Looping sound effect entity that follows a black hole.
#[derive(Component)]
pub(super) struct BlackHoleSfx {
    /// The parent black hole entity this sound tracks.
    pub black_hole_entity: Entity,
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
