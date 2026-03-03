use bevy::prelude::*;

use super::constants;
use crate::game::units::wizard::spells::utils::{CircleIndicator, indicator_pulse_scale};

/// Visual indicator for the Entangle area during casting.
#[derive(Component)]
pub struct EntangleIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl EntangleIndicator {
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }
}

impl CircleIndicator for EntangleIndicator {
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

/// Persistent ground visual showing the entangle area.
#[derive(Component)]
pub struct EntangleGroundEffect {
    pub time_remaining: f32,
    pub duration: f32,
}

impl EntangleGroundEffect {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
            duration,
        }
    }
}
