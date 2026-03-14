use bevy::prelude::*;

use super::constants;

/// Bubble particle that rises from the grease surface and pops.
#[derive(Component)]
pub(crate) struct GreaseBubble {
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before pop/despawn.
    pub lifetime: f32,
    /// Maximum scale at pop moment.
    pub max_size: f32,
    /// Upward velocity.
    pub rise_speed: f32,
}

/// Splatter particle that appears at zone edges and fades out.
#[derive(Component)]
pub(crate) struct GreaseSplatter {
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
}

/// Talent parameters computed from active talent selections.
/// Stored on each GreaseZone entity so talent logic can reference it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GreaseTalentParams {
    // Tier 1: numeric modifiers
    pub radius_mult: f32,
    pub slow_mult: f32,
    pub burn_damage_mult: f32,
    // Tier 2: behavioral flags
    pub slip_and_fall: bool,
    pub oil_slick: bool,
    pub lingering_flames: bool,
    // Tier 3: transformative flags
    pub chain_combustion: bool,
    pub grease_geyser: bool,
    pub endless_oil: bool,
}

impl Default for GreaseTalentParams {
    fn default() -> Self {
        Self {
            radius_mult: 1.0,
            slow_mult: 1.0,
            burn_damage_mult: 1.0,
            slip_and_fall: false,
            oil_slick: false,
            lingering_flames: false,
            chain_combustion: false,
            grease_geyser: false,
            endless_oil: false,
        }
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
    pub empowerment: f32,
    /// Pre-configured ignition damage (applied when GreaseIgnited is inserted).
    pub ignite_damage: f32,
    /// Pre-configured burn damage per tick.
    pub ignite_burn_damage: f32,
    /// Pre-configured burn tick interval.
    pub ignite_burn_tick: f32,
    /// Talent params for this specific cast.
    pub talent_params: GreaseTalentParams,
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
        talent_params: GreaseTalentParams,
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
            talent_params,
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

/// Tracks a unit's presence inside a grease zone.
/// Used by Slip and Fall (stun on entry) and Oil Slick (armor debuff tracking).
#[derive(Component)]
pub(crate) struct GreaseZonePresenceTracker {
    /// Entity ID of the zone this tracker belongs to.
    pub zone_entity: Entity,
}

/// Armor debuff applied by the Oil Slick talent.
/// Increases spell damage taken while inside the grease zone.
#[derive(Component)]
pub(crate) struct GreaseOilSlickDebuff {
    /// The vulnerability amount applied to the unit's health.
    pub vulnerability: f32,
}

impl GreaseOilSlickDebuff {
    pub fn new() -> Self {
        Self {
            vulnerability: constants::OIL_SLICK_VULNERABILITY,
        }
    }
}

/// Marker for a grease zone that is regenerating after being ignited (Endless Oil talent).
/// The zone returns to its slippery state and can be re-ignited.
#[derive(Component)]
pub(crate) struct GreaseRegenerating {
    /// Time spent regenerating so far.
    pub time_regenerating: f32,
}

impl GreaseRegenerating {
    pub fn new() -> Self {
        Self {
            time_regenerating: 0.0,
        }
    }
}

