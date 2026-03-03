use bevy::prelude::*;

use super::constants;
use crate::game::units::wizard::spells::utils::{CircleIndicator, indicator_pulse_scale};

/// Visual indicator for the Haste area during casting.
#[derive(Component)]
pub struct HasteIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl HasteIndicator {
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }
}

impl CircleIndicator for HasteIndicator {
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
        indicator_pulse_scale(self.time_alive)
    }
}
