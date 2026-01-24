use bevy::prelude::*;

use super::constants;

/// Component for magic missile projectiles.
///
/// Magic missiles lock onto a target when launched and track it until it despawns.
#[derive(Component)]
pub struct MagicMissile {
    /// Current velocity of the missile.
    pub velocity: Vec3,
    /// Initial homing strength (increases over time).
    pub base_homing_strength: f32,
    /// Damage dealt on impact.
    pub damage: f32,
    /// Collision radius.
    pub radius: f32,
    /// Accumulated time for path variability.
    pub time_alive: f32,
    /// Random offset for this specific missile's wobble pattern.
    pub wobble_offset: f32,
    /// Locked target entity (retargets only if this despawns).
    pub target: Option<Entity>,
}

impl MagicMissile {
    /// Creates a new magic missile.
    ///
    /// # Arguments
    ///
    /// * `initial_velocity` - Starting velocity vector
    /// * `wobble_offset` - Random offset for wobble pattern
    /// * `target` - Initial target entity to lock onto
    pub fn new(initial_velocity: Vec3, wobble_offset: f32, target: Option<Entity>) -> Self {
        Self {
            velocity: initial_velocity,
            base_homing_strength: constants::BASE_HOMING_STRENGTH,
            damage: constants::DAMAGE,
            radius: constants::COLLISION_RADIUS,
            time_alive: 0.0,
            wobble_offset,
            target,
        }
    }

    /// Calculates current homing strength based on time alive.
    ///
    /// Homing increases over perfect tracking time, then becomes perfect tracking.
    pub fn current_homing_strength(&self) -> f32 {
        if self.time_alive >= constants::PERFECT_TRACKING_TIME {
            // After perfect tracking time, return effectively infinite homing (perfect tracking)
            f32::INFINITY
        } else {
            // Ramp up over perfect tracking time
            let t = self.time_alive / constants::PERFECT_TRACKING_TIME;
            let strength_multiplier = t * constants::HOMING_RAMP_MULTIPLIER;
            self.base_homing_strength * (1.0 + strength_multiplier)
        }
    }

    /// Calculates current max speed based on time alive.
    ///
    /// Speed increases based on multipliers over perfect tracking time.
    pub fn current_max_speed(&self) -> f32 {
        if self.time_alive >= constants::PERFECT_TRACKING_TIME {
            // After perfect tracking time, max speed reaches final multiplier
            constants::BASE_SPEED * constants::FINAL_SPEED_MULTIPLIER
        } else {
            // Ramp up from 1x to final multiplier over perfect tracking time
            let t = self.time_alive / constants::PERFECT_TRACKING_TIME;
            constants::BASE_SPEED * (1.0 + t * constants::SPEED_RAMP_MULTIPLIER)
        }
    }
}
