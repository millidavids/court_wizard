use bevy::prelude::*;

use super::constants;
use crate::game::units::wizard::spells::utils::{CircleIndicator, indicator_pulse_scale};

/// Visual indicator for the Guardian Circle area during casting.
///
/// Shows the area of effect that will receive temporary hit points.
#[derive(Component)]
pub struct GuardianCircleIndicator {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
    /// Whether this circle is empowered.
    pub empowerment: f32,
}

impl GuardianCircleIndicator {
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
        indicator_pulse_scale(self.time_alive)
    }
}

impl CircleIndicator for GuardianCircleIndicator {
    fn position(&self) -> Vec3 {
        self.position
    }
    fn time_alive(&self) -> f32 {
        self.time_alive
    }
    fn set_time_alive(&mut self, time: f32) {
        self.time_alive = time;
    }
    fn base_radius(&self) -> f32 {
        constants::CIRCLE_RADIUS * self.empowerment
    }
    fn circle_y_position(&self) -> f32 {
        constants::CIRCLE_Y_POSITION
    }
    fn pulse_scale(&self) -> f32 {
        self.pulse_scale()
    }
}
