use bevy::prelude::*;

#[derive(Component)]
pub struct PhantasmalForceIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl PhantasmalForceIndicator {
    pub fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
        }
    }

    pub fn pulse_scale(&self) -> f32 {
        1.0 + (self.time_alive * 2.0 * std::f32::consts::TAU).sin() * 0.05
    }
}
