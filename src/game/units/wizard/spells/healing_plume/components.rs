use std::collections::HashSet;
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

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct HealingPlumeTalentParams {
    // Tier 1: numeric modifiers
    /// Rejuvenating Mists: multiplier on healing per tick.
    pub heal_mult: f32,
    /// Verdant Bloom: multiplier on zone radius.
    pub radius_mult: f32,
    /// Lasting Remedy: multiplier on zone duration.
    pub duration_mult: f32,
    // Tier 2: behavioral flags
    /// Cleansing Plume: remove debuffs from allies in zone.
    pub cleansing_plume: bool,
    /// Overflow: excess healing becomes temp HP.
    pub overflow: bool,
    /// Triage Pulse: double healing for low-HP allies.
    pub triage_pulse: bool,
    // Tier 3: transformative flags
    /// Font of Life: resurrect units that die in the zone.
    pub font_of_life: bool,
    /// Healing Rain: zone follows the cursor.
    pub healing_rain: bool,
    /// Field Medic: convert a defender into a temporary healer.
    pub field_medic: bool,
}

impl Default for HealingPlumeTalentParams {
    fn default() -> Self {
        Self {
            heal_mult: 1.0,
            radius_mult: 1.0,
            duration_mult: 1.0,
            cleansing_plume: false,
            overflow: false,
            triage_pulse: false,
            font_of_life: false,
            healing_rain: false,
            field_medic: false,
        }
    }
}

/// Tier 2: Cleansing Plume — marks a zone as cleansing-enabled.
/// Tracked on the zone entity itself.
#[derive(Component)]
pub(crate) struct CleansingPlumeZone {
    pub time_since_last_cleanse: f32,
}

impl CleansingPlumeZone {
    pub const fn new() -> Self {
        Self {
            time_since_last_cleanse: 0.0,
        }
    }
}

/// Tier 2: Overflow — marks a zone as granting temp HP from excess healing.
#[derive(Component)]
pub(crate) struct OverflowZone;

/// Tier 2: Triage Pulse — marks a zone as providing double healing below threshold.
#[derive(Component)]
pub(crate) struct TriagePulseZone;

/// Tier 3: Font of Life — marks a zone that can resurrect units that die inside.
/// Tracks which entities have already been resurrected (once per unit).
#[derive(Component)]
pub(crate) struct FontOfLifeZone {
    /// Entities that have already been resurrected by this zone.
    pub resurrected: HashSet<Entity>,
}

impl FontOfLifeZone {
    pub fn new() -> Self {
        Self {
            resurrected: HashSet::new(),
        }
    }
}

/// Tier 3: Font of Life — pending resurrection timer on a corpse.
#[derive(Component)]
pub(crate) struct FontOfLifePending {
    /// Time remaining before resurrection.
    pub time_remaining: f32,
}

/// Tier 3: Healing Rain — marks a zone as mobile (follows cursor).
#[derive(Component)]
pub(crate) struct HealingRainZone;

/// The original unit type before Field Medic conversion.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldMedicOriginalType {
    Infantry,
    Archer,
}

/// Tier 3: Field Medic — marker on the converted unit, storing original state.
#[derive(Component)]
pub(crate) struct FieldMedicConverted {
    /// The zone entity that created this conversion.
    pub zone_entity: Entity,
    /// The original unit type so we can restore it.
    pub original_type: FieldMedicOriginalType,
    /// The original material handle so we can restore the unit's appearance.
    pub original_material: Handle<StandardMaterial>,
}
