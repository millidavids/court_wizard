use bevy::prelude::*;

use crate::game::units::wizard::spells::utils::indicator_pulse_scale;

#[derive(Component)]
pub struct BattleHymnIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
    /// Talent-based radius multiplier (e.g., 1.4 for Wide Anthem).
    pub talent_radius_mult: f32,
}

impl BattleHymnIndicator {
    pub fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
            talent_radius_mult: 1.0,
        }
    }

    pub fn pulse_scale(&self) -> f32 {
        indicator_pulse_scale(self.time_alive)
    }
}
