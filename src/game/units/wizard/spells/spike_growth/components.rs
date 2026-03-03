use bevy::prelude::*;

use crate::game::units::wizard::spells::utils::indicator_pulse_scale;

/// Visual indicator for the Spike Growth area during casting.
#[derive(Component)]
pub struct SpikeGrowthIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl SpikeGrowthIndicator {
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

/// Persistent spike growth zone that damages and slows ALL units inside.
#[derive(Component)]
pub struct SpikeGrowthZone {
    pub origin: Vec3,
    pub radius: f32,
    pub damage_per_tick: f32,
    pub tick_interval: f32,
    pub slow_modifier: f32,
    pub slow_duration: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub time_since_last_tick: f32,
}

impl SpikeGrowthZone {
    pub fn new(
        origin: Vec3,
        radius: f32,
        damage_per_tick: f32,
        tick_interval: f32,
        slow_modifier: f32,
        slow_duration: f32,
        duration: f32,
    ) -> Self {
        Self {
            origin,
            radius,
            damage_per_tick,
            tick_interval,
            slow_modifier,
            slow_duration,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
        }
    }
}
