use bevy::prelude::*;

/// Persistent healing zone that heals all units inside.
#[derive(Component)]
pub struct HealingPlumeZone {
    pub origin: Vec3,
    pub radius: f32,
    pub heal_per_tick: f32,
    pub tick_interval: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub time_since_last_tick: f32,
}

impl HealingPlumeZone {
    pub fn new(
        origin: Vec3,
        radius: f32,
        heal_per_tick: f32,
        tick_interval: f32,
        duration: f32,
    ) -> Self {
        Self {
            origin,
            radius,
            heal_per_tick,
            tick_interval,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
        }
    }
}
