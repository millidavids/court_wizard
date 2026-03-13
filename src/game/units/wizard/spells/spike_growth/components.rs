use bevy::prelude::*;

use super::constants;

/// Talent parameters computed from active talent selections.
/// Stored on each SpikeGrowthZone entity so talent logic can reference it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SpikeGrowthTalentParams {
    // Tier 1: numeric modifiers
    pub radius_mult: f32,
    pub damage_mult: f32,
    pub mana_mult: f32,
    pub cast_time_mult: f32,
    // Tier 2: behavioral flags
    pub thorn_maze: bool,
    pub poisoned_spikes: bool,
    pub quicksand: bool,
    // Tier 3: transformative flags
    pub death_garden: bool,
    pub minefield: bool,
    pub spike_storm: bool,
}

impl Default for SpikeGrowthTalentParams {
    fn default() -> Self {
        Self {
            radius_mult: 1.0,
            damage_mult: 1.0,
            mana_mult: 1.0,
            cast_time_mult: 1.0,
            thorn_maze: false,
            poisoned_spikes: false,
            quicksand: false,
            death_garden: false,
            minefield: false,
            spike_storm: false,
        }
    }
}

/// Persistent spike growth zone that damages and slows ALL units inside.
#[derive(Component)]
pub struct SpikeGrowthZone {
    pub origin: Vec3,
    pub base_radius: f32,
    pub damage_per_tick: f32,
    pub tick_interval: f32,
    pub slow_modifier: f32,
    pub slow_duration: f32,
    pub duration: f32,
    pub time_alive: f32,
    pub time_since_last_tick: f32,
    /// Talent params for this specific cast.
    pub talent_params: SpikeGrowthTalentParams,
    /// Death Garden: extra duration accumulated from kills.
    pub death_garden_extension: f32,
    /// Spike Storm: timer for projectile volleys.
    pub spike_storm_timer: f32,
    /// Spike Storm: cached material handle for projectiles.
    pub spike_storm_material: Option<Handle<StandardMaterial>>,
    /// Death Garden: timer for obstacle update throttling.
    pub death_garden_obstacle_timer: f32,
    /// Timer for animated ring VFX spawning.
    pub ring_timer: f32,
}

impl SpikeGrowthZone {
    pub fn new(
        origin: Vec3,
        base_radius: f32,
        damage_per_tick: f32,
        tick_interval: f32,
        slow_modifier: f32,
        slow_duration: f32,
        duration: f32,
        talent_params: SpikeGrowthTalentParams,
    ) -> Self {
        Self {
            origin,
            base_radius,
            damage_per_tick,
            tick_interval,
            slow_modifier,
            slow_duration,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
            talent_params,
            death_garden_extension: 0.0,
            spike_storm_timer: 0.0,
            spike_storm_material: None,
            death_garden_obstacle_timer: 0.0,
            ring_timer: 0.0,
        }
    }

    /// Effective duration including Death Garden extensions.
    pub fn effective_duration(&self) -> f32 {
        self.duration + self.death_garden_extension
    }

    /// Current effective radius accounting for Death Garden growth.
    pub fn effective_radius(&self) -> f32 {
        if self.talent_params.death_garden {
            let progress = (self.time_alive / self.effective_duration()).min(1.0);
            self.base_radius * (1.0 + constants::DEATH_GARDEN_GROWTH_FRACTION * progress)
        } else {
            self.base_radius
        }
    }
}

/// Tracks a unit's presence inside a spike growth zone.
/// Used by Quicksand (root after threshold) and Poisoned Spikes (lingering on exit).
#[derive(Component)]
pub(crate) struct ZonePresenceTracker {
    /// Entity ID of the zone this tracker belongs to.
    pub zone_entity: Entity,
    /// Time spent inside the zone (used by Quicksand).
    pub time_in_zone: f32,
    /// Whether the quicksand root has already been applied (once per unit per zone).
    pub rooted: bool,
}

/// Lingering poison effect applied to units that leave a Poisoned Spikes zone.
#[derive(Component)]
pub(crate) struct SpikeGrowthLingeringPoison {
    /// Damage per tick.
    pub damage_per_tick: f32,
    /// Tick interval.
    pub tick_interval: f32,
    /// Time remaining.
    pub time_remaining: f32,
    /// Time since last tick.
    pub time_since_last_tick: f32,
}

impl SpikeGrowthLingeringPoison {
    pub fn new() -> Self {
        Self {
            damage_per_tick: constants::POISONED_SPIKES_LINGER_DPS
                * constants::POISONED_SPIKES_LINGER_TICK_INTERVAL,
            tick_interval: constants::POISONED_SPIKES_LINGER_TICK_INTERVAL,
            time_remaining: constants::POISONED_SPIKES_LINGER_DURATION,
            time_since_last_tick: 0.0,
        }
    }
}

/// Spike Storm projectile launched from a spike growth zone.
#[derive(Component)]
pub(crate) struct SpikeStormProjectile {
    pub direction: Vec3,
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
    pub time_alive: f32,
    pub max_lifetime: f32,
}

