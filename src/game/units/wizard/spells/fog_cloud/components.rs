use bevy::prelude::*;

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
    /// Timer for spawning smoke particles.
    pub smoke_spawn_timer: f32,
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
            smoke_spawn_timer: 0.0,
        }
    }
}

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct FogCloudTalentParams {
    // Tier 1: numeric modifiers
    pub evasion_chance: f32,
    pub radius_mult: f32,
    pub linger_duration: f32,
    // Tier 2: behavioral flags
    pub blinding_mist: bool,
    pub concealing_veil: bool,
    pub disorienting_vapors: bool,
    // Tier 3: transformative flags
    pub phantom_fog: bool,
    pub choking_fog: bool,
    pub rolling_fog: bool,
}

impl Default for FogCloudTalentParams {
    fn default() -> Self {
        Self {
            evasion_chance: super::constants::EVASION_CHANCE,
            radius_mult: 1.0,
            linger_duration: super::constants::EVASION_REFRESH_DURATION,
            blinding_mist: false,
            concealing_veil: false,
            disorienting_vapors: false,
            phantom_fog: false,
            choking_fog: false,
            rolling_fog: false,
        }
    }
}

/// Tier 2: Units inside the fog have their attack range halved.
#[derive(Component)]
pub(crate) struct BlindingMistZone;

/// Per-unit debuff: attack range halved while in fog.
#[derive(Component)]
pub(crate) struct BlindingMistDebuff {
    pub time_remaining: f32,
    pub range_mult: f32,
}

impl BlindingMistDebuff {
    pub fn new(range_mult: f32) -> Self {
        Self {
            time_remaining: super::constants::BLINDING_MIST_DEBUFF_DURATION,
            range_mult,
        }
    }

    pub fn refresh(&mut self) {
        self.time_remaining = super::constants::BLINDING_MIST_DEBUFF_DURATION;
    }
}

/// Tier 2: Allies inside the fog cannot be targeted by ranged attacks from outside.
#[derive(Component)]
pub(crate) struct ConcealingVeilZone;

/// Tier 2: 20% chance units attack an ally instead of their target.
#[derive(Component)]
pub(crate) struct DisorientingVaporsZone;

/// Tier 3: Phantom fog — periodically spawns phantom decoy units inside the fog.
#[derive(Component)]
pub(crate) struct PhantomFogZone {
    pub spawn_timer: f32,
}

/// Marker for phantom decoy units spawned by the Phantom Fog talent.
/// Phantoms are targetable but don't attack, die in one hit, and leave no corpse.
#[derive(Component)]
pub(crate) struct PhantomUnit;

/// Tier 3: Fog deals minor damage per second to non-ally units.
#[derive(Component)]
pub(crate) struct ChokingFogZone {
    pub dps: f32,
    pub tick_interval: f32,
    pub tick_accumulator: f32,
}

impl ChokingFogZone {
    pub fn new(dps: f32, tick_interval: f32) -> Self {
        Self {
            dps,
            tick_interval,
            tick_accumulator: 0.0,
        }
    }
}

/// Tier 3: Fog slowly moves toward incoming attackers.
#[derive(Component)]
pub(crate) struct RollingFogZone {
    pub speed: f32,
}
