use bevy::prelude::*;

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
    pub empowerment: f32,
    /// Pre-configured ignition damage (applied when GreaseIgnited is inserted).
    pub ignite_damage: f32,
    /// Pre-configured burn damage per tick.
    pub ignite_burn_damage: f32,
    /// Pre-configured burn tick interval.
    pub ignite_burn_tick: f32,
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
            empowerment,
            ignite_damage,
            ignite_burn_damage,
            ignite_burn_tick,
        }
    }
}

/// Marker component inserted when a grease zone is set on fire.
/// Drives burn damage, fire spread VFX, and prevents normal time_alive tracking.
#[derive(Component)]
pub struct GreaseIgnited {
    /// XZ point where fire started spreading from.
    pub ignition_point: Vec3,
    /// Time since ignition occurred (for fire spread animation).
    pub fire_spread_time: f32,
}

impl GreaseIgnited {
    pub fn new(ignition_point: Vec3) -> Self {
        Self {
            ignition_point,
            fire_spread_time: 0.0,
        }
    }

    /// Returns the current fire spread radius based on time since ignition.
    pub fn current_fire_radius(&self, zone_radius: f32, spread_duration: f32) -> f32 {
        let progress = (self.fire_spread_time / spread_duration).min(1.0);
        zone_radius * progress
    }
}

/// Fire overlay mesh that visually spreads from the ignition point.
#[derive(Component)]
pub struct GreaseFireOverlay {
    pub zone_entity: Entity,
}
