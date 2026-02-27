use bevy::prelude::*;

use super::constants;
use crate::game::units::DamageType;

/// Marker for the outer glow cylinder that follows a beam.
#[derive(Component)]
pub struct BeamGlow {
    /// The beam entity this glow tracks.
    pub beam_entity: Entity,
}

/// Marker for the bright flare sphere at a beam's origin.
#[derive(Component)]
pub struct BeamOriginFlare {
    /// The beam entity this flare tracks.
    pub beam_entity: Entity,
}

/// A small particle emitted from the beam's impact point.
#[derive(Component)]
pub struct DisintegrateParticle {
    /// World-space velocity of the particle.
    pub velocity: Vec3,
    /// Seconds since this particle was spawned.
    pub time_alive: f32,
}

/// A smoke wisp that drifts upward off the beam and self-dissipates.
/// These are independent entities that persist after the beam despawns.
#[derive(Component)]
pub struct BeamSmoke {
    /// World-space velocity (primarily upward with slight lateral spread).
    pub velocity: Vec3,
    /// Seconds since this wisp was spawned.
    pub time_alive: f32,
}

/// Component for disintegrate beam.
///
/// The beam is a continuous ray that deals damage to entities along its path.
#[derive(Component)]
pub struct DisintegrateBeam {
    /// Origin point of the beam in world space.
    pub origin: Vec3,
    /// Direction the beam is pointing (normalized).
    pub direction: Vec3,
    /// Length of the beam.
    pub length: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time since last damage tick.
    pub time_since_damage: f32,
    /// Time since beam was spawned (used for growth animation).
    pub time_alive: f32,
    /// Whether this beam is empowered.
    pub empowerment: f32,
    /// Optional damage override (used by crystal beams with custom damage).
    pub damage_per_tick_override: Option<f32>,
}

impl DisintegrateBeam {
    /// Creates a new disintegrate beam.
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting position of the beam
    /// * `direction` - Direction the beam points (will be normalized)
    /// * `length` - Length of the beam
    /// * `empowerment` - Empowerment multiplier
    pub fn new(origin: Vec3, direction: Vec3, length: f32, empowerment: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            damage_type: constants::DAMAGE_TYPE,
            time_since_damage: 0.0,
            time_alive: 0.0,
            empowerment,
            damage_per_tick_override: None,
        }
    }

    /// Gets the damage per tick, using override if set, otherwise scaled by empowerment.
    pub fn damage_per_tick(&self) -> f32 {
        if let Some(override_damage) = self.damage_per_tick_override {
            override_damage * self.empowerment
        } else {
            constants::DAMAGE_PER_TICK * self.empowerment
        }
    }

    /// Gets the beam width, scaled by empowerment.
    pub fn beam_width(&self) -> f32 {
        let scale = self.empowerment;
        constants::BEAM_WIDTH * scale
    }

    /// Checks if enough time has passed to deal damage again.
    pub fn should_damage(&self) -> bool {
        self.time_since_damage >= constants::DAMAGE_INTERVAL
    }

    /// Resets the damage timer.
    pub fn reset_damage_timer(&mut self) {
        self.time_since_damage = 0.0;
    }

    /// Updates the damage timer.
    pub fn update_damage_timer(&mut self, delta: f32) {
        self.time_since_damage += delta;
    }

    /// Updates the time alive counter.
    pub fn update_time_alive(&mut self, delta: f32) {
        self.time_alive += delta;
    }

    /// Gets the current animated length based on growth time.
    ///
    /// Beam grows from 0 to full length over BEAM_GROWTH_TIME seconds.
    pub fn current_length(&self) -> f32 {
        if self.time_alive >= constants::BEAM_GROWTH_TIME {
            self.length
        } else {
            let growth_factor = self.time_alive / constants::BEAM_GROWTH_TIME;
            self.length * growth_factor
        }
    }

    /// Checks if a point is within the beam.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to check
    ///
    /// # Returns
    ///
    /// True if the point is within the beam's width and length.
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.contains_point_with_radius(point, 0.0)
    }

    /// Checks if a unit with the given hitbox radius is hit by the beam.
    ///
    /// The beam hits if the distance from the beam centerline to the unit center
    /// is less than beam_width + unit_radius.
    pub fn contains_point_with_radius(&self, point: Vec3, unit_radius: f32) -> bool {
        let to_point = point - self.origin;
        let projection_length = to_point.dot(self.direction);

        // Check if point is within current animated beam length (accounting for unit radius)
        let current_len = self.current_length();
        if projection_length < -unit_radius || projection_length > current_len + unit_radius {
            return false;
        }

        // Check distance from beam centerline
        let closest_point_on_beam =
            self.origin + self.direction * projection_length.clamp(0.0, current_len);
        let distance_from_beam = point.distance(closest_point_on_beam);

        distance_from_beam <= self.beam_width() + unit_radius
    }
}
