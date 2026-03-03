use bevy::prelude::*;

use crate::game::units::wizard::spells::utils::indicator_pulse_scale;

#[derive(Component)]
pub struct BerserkerRageIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl BerserkerRageIndicator {
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }

    pub fn pulse_scale(&self) -> f32 {
        indicator_pulse_scale(self.time_alive)
    }
}
