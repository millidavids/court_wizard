use bevy::prelude::*;

#[derive(Component)]
pub struct GreaseIndicator {
    pub position: Vec3,
    pub time_alive: f32,
    pub empowerment: f32,
}

impl GreaseIndicator {
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
pub struct GreaseZone {
    pub origin: Vec3,
    pub radius: f32,
    pub slow_modifier: f32,
    pub slow_duration: f32,
    pub tick_interval: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub time_since_last_tick: f32,
    pub ignited: bool,
    pub ignite_damage: f32,
    pub ignite_burn_damage: f32,
    pub ignite_burn_tick: f32,
    pub empowerment: f32,
    /// XZ point where fire started spreading from
    pub ignition_point: Option<Vec3>,
    /// Time since ignition occurred (for fire spread animation)
    pub fire_spread_time: f32,
}

impl GreaseZone {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: Vec3,
        radius: f32,
        slow_modifier: f32,
        slow_duration: f32,
        tick_interval: f32,
        duration: f32,
        ignite_damage: f32,
        ignite_burn_damage: f32,
        ignite_burn_tick: f32,
        empowerment: f32,
    ) -> Self {
        Self {
            origin,
            radius,
            slow_modifier,
            slow_duration,
            tick_interval,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
            ignited: false,
            ignite_damage,
            ignite_burn_damage,
            ignite_burn_tick,
            empowerment,
            ignition_point: None,
            fire_spread_time: 0.0,
        }
    }

    /// Returns the current fire spread radius based on time since ignition.
    pub fn current_fire_radius(&self, spread_duration: f32) -> f32 {
        if !self.ignited {
            return 0.0;
        }
        let progress = (self.fire_spread_time / spread_duration).min(1.0);
        self.radius * progress
    }
}

/// Fire overlay mesh that visually spreads from the ignition point.
#[derive(Component)]
pub struct GreaseFireOverlay {
    pub zone_entity: Entity,
}
