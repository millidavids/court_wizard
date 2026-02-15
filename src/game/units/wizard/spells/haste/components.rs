use bevy::prelude::*;

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

    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}
