use bevy::prelude::Color;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_HEALING_PLUME: PrimedSpell = PrimedSpell {
    spell: Spell::HealingPlume,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 30.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const HEAL_PER_TICK: f32 = 4.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const ZONE_DURATION: f32 = 10.0;
pub const FADE_DURATION: f32 = 2.0;

// ===== Talent Constants =====

// Tier 1
/// Rejuvenating Mists: healing per tick multiplier (+40%).
pub const REJUVENATING_MISTS_HEAL_MULT: f32 = 1.4;
/// Verdant Bloom: radius multiplier (+40%).
pub const VERDANT_BLOOM_RADIUS_MULT: f32 = 1.4;
/// Lasting Remedy: duration multiplier (+50%).
pub const LASTING_REMEDY_DURATION_MULT: f32 = 1.5;

// Tier 2
/// Cleansing Plume: interval between debuff cleanses (seconds).
pub const CLEANSING_PLUME_INTERVAL: f32 = 1.0;
/// Overflow: maximum temporary HP that can be granted.
pub const OVERFLOW_MAX_TEMP_HP: f32 = 20.0;
/// Overflow: duration of the temporary HP (seconds).
pub const OVERFLOW_TEMP_HP_DURATION: f32 = 15.0;
/// Triage Pulse: HP threshold below which healing is doubled.
pub const TRIAGE_PULSE_HP_THRESHOLD: f32 = 0.3;
/// Triage Pulse: healing multiplier when below threshold.
pub const TRIAGE_PULSE_HEAL_MULT: f32 = 2.0;

// Tier 3
/// Font of Life: HP percentage to resurrect at.
pub const FONT_OF_LIFE_RESURRECT_HP_PERCENT: f32 = 0.25;
/// Font of Life: delay before resurrection (seconds).
pub const FONT_OF_LIFE_RESURRECT_DELAY: f32 = 3.0;
/// Font of Life: movement speed of resurrected units.
pub const FONT_OF_LIFE_RESURRECT_SPEED: f32 = 80.0;
/// Healing Rain: healing reduction multiplier (75% of base healing).
pub const HEALING_RAIN_HEAL_MULT: f32 = 0.75;
/// Healing Rain: movement speed of the zone toward cursor (units/sec).
pub const HEALING_RAIN_MOVE_SPEED: f32 = 40.0;
/// Field Medic: green tint color for converted unit.
pub const FIELD_MEDIC_COLOR: Color = Color::srgba(0.3, 0.85, 0.3, 1.0);
