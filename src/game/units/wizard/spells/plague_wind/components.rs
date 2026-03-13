use bevy::prelude::*;

/// Marker component for the plague wind casting indicator.
/// Stores the directional arrow entity; position/radius/pulse are handled
/// by the shared `SpellCircleIndicator` component.
#[derive(Component)]
pub struct PlagueWindIndicator {
    /// Directional arrow entity showing wind direction.
    pub arrow_entity: Option<Entity>,
}


/// Pre-computed talent parameters for a plague wind cloud.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlagueWindTalentParams {
    // Tier 1: numeric modifiers
    pub damage_mult: f32,
    pub radius_mult: f32,
    pub duration_mult: f32,
    pub speed_mult: f32,
    // Tier 2: behavioral flags
    pub plague_carrier: bool,
    pub toxic_weakness: bool,
    pub choking_gas: bool,
    // Tier 3: transformative flags
    pub pandemic: bool,
    pub twin_plumes: bool,
    pub necrotic_rot: bool,
}

impl Default for PlagueWindTalentParams {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            radius_mult: 1.0,
            duration_mult: 1.0,
            speed_mult: 1.0,
            plague_carrier: false,
            toxic_weakness: false,
            choking_gas: false,
            pandemic: false,
            twin_plumes: false,
            necrotic_rot: false,
        }
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
    pub talent_params: PlagueWindTalentParams,
    /// Timer for spawning smoke particles.
    pub smoke_spawn_timer: f32,
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
        talent_params: PlagueWindTalentParams,
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
            talent_params,
            smoke_spawn_timer: 0.0,
        }
    }
}

/// Marks a unit that recently left a plague wind cloud and still takes lingering damage.
/// Used by the Plague Carrier talent (Tier 2-0).
#[derive(Component)]
pub(crate) struct PlagueCarrierDoT {
    pub damage_per_tick: f32,
    pub tick_interval: f32,
    pub time_remaining: f32,
    pub time_since_last_tick: f32,
}

impl PlagueCarrierDoT {
    pub fn new(damage_per_tick: f32, tick_interval: f32, duration: f32) -> Self {
        Self {
            damage_per_tick,
            tick_interval,
            time_remaining: duration,
            time_since_last_tick: 0.0,
        }
    }
}

/// Tracks which entities are currently inside a plague wind cloud.
/// Used by the Plague Carrier talent to detect when units leave.
#[derive(Component)]
pub(crate) struct InsidePlagueCloud;

/// Marks units affected by Toxic Weakness so the debuff can be removed when they leave.
/// Stores the vulnerability amount added so it can be subtracted cleanly.
#[derive(Component)]
pub(crate) struct ToxicWeaknessDebuff(pub f32);

/// Marks a dead unit that has already spawned a Pandemic child cloud.
/// Prevents spawning multiple children from the same death.
#[derive(Component)]
pub(crate) struct PandemicProcessed;

