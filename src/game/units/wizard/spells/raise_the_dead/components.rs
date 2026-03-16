use bevy::prelude::*;

/// Marker component for units that have been raised from the dead
#[derive(Component)]
pub struct RaisedUndead;

/// Computed talent parameters for Raise The Dead casting.
/// Not a Component — used transiently during casting logic.
#[derive(Clone)]
pub(crate) struct RaiseTheDeadTalentParams {
    // Tier 1: numeric modifiers
    pub radius_mult: f32,
    pub skip_ramp: bool,
    pub mana_cost_mult: f32,

    // Tier 2: behavioral flags
    pub empowered_undead: bool,
    pub plague_bearer: bool,
    pub corpse_magnet: bool,

    // Tier 3: transformative flags
    pub revenant_lord: bool,
    pub undead_detonation: bool,
    pub perpetual_unrest: bool,
}

impl Default for RaiseTheDeadTalentParams {
    fn default() -> Self {
        Self {
            radius_mult: 1.0,
            skip_ramp: false,
            mana_cost_mult: 1.0,
            empowered_undead: false,
            plague_bearer: false,
            corpse_magnet: false,
            revenant_lord: false,
            undead_detonation: false,
            perpetual_unrest: false,
        }
    }
}

/// Tier 2: Plague Bearer — raised undead emit a poison aura.
/// Ticks damage to nearby living enemies.
#[derive(Component)]
pub struct PlagueBearerAura {
    pub dps: f32,
    pub radius: f32,
    pub tick_accumulator: f32,
    pub tick_interval: f32,
    pub smoke_spawn_timer: f32,
}

impl PlagueBearerAura {
    pub fn new(dps: f32, radius: f32, tick_interval: f32) -> Self {
        Self {
            dps,
            radius,
            tick_accumulator: 0.0,
            tick_interval,
            smoke_spawn_timer: 0.0,
        }
    }
}

/// Tier 2: Corpse Magnet — active during channeling, pulls corpses toward cursor.
/// Attached to the wizard entity during channeling.
#[derive(Component)]
pub struct CorpseMagnetActive {
    pub pull_radius: f32,
    pub pull_speed: f32,
}

/// Tier 3: Undead Detonation — raised undead explode on death.
/// Marker component attached to raised undead.
#[derive(Component)]
pub struct UndeadDetonation {
    pub damage: f32,
    pub radius: f32,
}

/// Tier 3: Perpetual Unrest — when this undead's kills cause auto-raises.
/// Marker component attached to raised undead.
#[derive(Component)]
pub struct PerpetualUnrest {
    pub raise_radius: f32,
}

/// Tier 3: Revenant Lord — marks the single powerful Revenant unit.
/// Passively resurrects nearby corpses as undead minions.
#[derive(Component)]
pub struct RevenantLord {
    pub raise_radius: f32,
    pub raise_interval: f32,
    pub raise_timer: f32,
}
