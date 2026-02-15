use bevy::prelude::*;

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

    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
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
