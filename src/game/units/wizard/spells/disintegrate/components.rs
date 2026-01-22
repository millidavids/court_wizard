use bevy::prelude::*;

use super::constants;

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
    /// Time since last damage tick.
    pub time_since_damage: f32,
    /// Time since beam was spawned (used for growth animation).
    pub time_alive: f32,
}

impl DisintegrateBeam {
    /// Creates a new disintegrate beam.
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting position of the beam
    /// * `direction` - Direction the beam points (will be normalized)
    /// * `length` - Length of the beam
    pub fn new(origin: Vec3, direction: Vec3, length: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            time_since_damage: 0.0,
            time_alive: 0.0,
        }
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
        let to_point = point - self.origin;
        let projection_length = to_point.dot(self.direction);

        // Check if point is within current animated beam length
        let current_len = self.current_length();
        if projection_length < 0.0 || projection_length > current_len {
            return false;
        }

        // Check distance from beam centerline
        let closest_point_on_beam = self.origin + self.direction * projection_length;
        let distance_from_beam = point.distance(closest_point_on_beam);

        distance_from_beam <= constants::BEAM_WIDTH
    }
}
