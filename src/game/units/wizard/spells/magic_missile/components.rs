use bevy::prelude::*;

/// Component for magic missile projectiles.
///
/// Magic missiles always seek the closest attacker with an arcing trajectory.
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
}

impl MagicMissile {
    /// Creates a new magic missile.
    ///
    /// # Arguments
    ///
    /// * `initial_velocity` - Starting velocity vector
    /// * `wobble_offset` - Random offset for wobble pattern
    pub fn new(initial_velocity: Vec3, wobble_offset: f32) -> Self {
        Self {
            velocity: initial_velocity,
            base_homing_strength: 400.0, // Starts low, increases over time
            damage: 50.0,
            radius: 10.0,
            time_alive: 0.0,
            wobble_offset,
        }
    }

    /// Calculates current homing strength based on time alive.
    ///
    /// Homing increases over 5 seconds, then becomes perfect tracking.
    pub fn current_homing_strength(&self) -> f32 {
        if self.time_alive >= 5.0 {
            // After 5 seconds, return effectively infinite homing (perfect tracking)
            f32::INFINITY
        } else {
            // Ramp up over 5 seconds
            let t = self.time_alive / 5.0; // 0.0 to 1.0 over 5 seconds
            let strength_multiplier = t * 19.0; // 0 to 19x multiplier
            self.base_homing_strength * (1.0 + strength_multiplier)
        }
    }

    /// Calculates current max speed based on time alive.
    ///
    /// Speed increases 3x over 5 seconds (600 -> 1800).
    pub fn current_max_speed(&self) -> f32 {
        let base_speed = 600.0;
        if self.time_alive >= 5.0 {
            // After 5 seconds, max speed is 3x
            base_speed * 3.0
        } else {
            // Ramp up from 1x to 3x over 5 seconds
            let t = self.time_alive / 5.0; // 0.0 to 1.0 over 5 seconds
            base_speed * (1.0 + t * 2.0) // 1.0x to 3.0x
        }
    }
}
