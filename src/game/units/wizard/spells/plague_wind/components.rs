use bevy::prelude::*;

/// Visual indicator during casting.
#[derive(Component)]
pub struct PlagueWindIndicator {
    pub position: Vec3,
    pub radius: f32,
    pub time_alive: f32,
}

impl PlagueWindIndicator {
    pub const fn new(position: Vec3, radius: f32) -> Self {
        Self {
            position,
            radius,
            time_alive: 0.0,
        }
    }

    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

/// Moving toxic cloud that drifts toward attacker spawn.
#[derive(Component)]
pub struct PlagueWindCloud {
    pub origin: Vec3,
    pub radius: f32,
    pub damage_per_tick: f32,
    pub tick_interval: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub time_since_last_tick: f32,
    pub speed: f32,
    pub direction: Vec3,
}

impl PlagueWindCloud {
    pub fn new(
        origin: Vec3,
        radius: f32,
        damage_per_tick: f32,
        tick_interval: f32,
        duration: f32,
        speed: f32,
        direction: Vec3,
    ) -> Self {
        Self {
            origin,
            radius,
            damage_per_tick,
            tick_interval,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
            speed,
            direction,
        }
    }
}
