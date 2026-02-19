use bevy::prelude::*;

#[derive(Component)]
pub struct FogCloudIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl FogCloudIndicator {
    pub const fn new(position: Vec3, empowerment: f32) -> Self {
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

#[derive(Component)]
pub struct FogCloudZone {
    pub origin: Vec3,
    pub radius: f32,
    pub evasion_chance: f32,
    pub evasion_refresh_duration: f32,
    pub tick_interval: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub time_since_last_tick: f32,
}

impl FogCloudZone {
    pub fn new(
        origin: Vec3,
        radius: f32,
        evasion_chance: f32,
        evasion_refresh_duration: f32,
        tick_interval: f32,
        duration: f32,
    ) -> Self {
        Self {
            origin,
            radius,
            evasion_chance,
            evasion_refresh_duration,
            tick_interval,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
        }
    }
}
