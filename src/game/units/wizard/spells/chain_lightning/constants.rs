use crate::game::units::DamageType;
use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_CHAIN_LIGHTNING: PrimedSpell = PrimedSpell {
    spell: Spell::ChainLightning,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// Casting
pub const CAST_TIME: f32 = 0.8;
pub const MANA_COST: f32 = 15.0;
pub const SPAWN_HEIGHT_OFFSET: f32 = 0.0;

// Damage
pub const INITIAL_DAMAGE: f32 = 10.0;
pub const DAMAGE_TYPE: DamageType = DamageType::Electric;
pub const DAMAGE_FALLOFF: f32 = 0.6;
pub const MAX_BOUNCES: u32 = 4;

// Splitting
pub const SPLIT_COUNT: usize = 2;

// Targeting
pub const TARGETING_RADIUS: f32 = 50.0; // Cursor proximity to enemy
pub const BOUNCE_RANGE: f32 = 100.0; // Max distance between targets

// Timing
pub const BOUNCE_DELAY: f32 = 0.02; // Time between bounces — almost instant, still perceptible
pub const ARC_LIFETIME: f32 = 0.3; // Arc visual persistence (active crackling phase)
pub const ARC_AFTERIMAGE_DURATION: f32 = 0.25; // Lingering shadow after the arc ends

// Visuals
pub const ARC_SEGMENTS: u32 = 16; // Number of jagged segments per arc
pub const ARC_HEIGHT_FACTOR: f32 = 0.15; // Base peak height as fraction of horizontal distance
pub const ARC_HEIGHT_GROWTH: f32 = 0.12; // Additional height factor per depth level
pub const ARC_JITTER_BASE: f32 = 12.0; // Perpendicular jitter amplitude at depth 0
pub const ARC_JITTER_DEPTH_FALLOFF: f32 = 0.7; // Jitter scales by this per depth level
pub const ARC_WIDTH: f32 = 8.0;
pub const ARC_WIDTH_FALLOFF: f32 = 0.8; // Width multiplier per depth level
pub const MIN_ARC_WIDTH: f32 = 2.0;

// ===== Talent Constants =====

// Tier 1
pub const CONDUCTING_BOLTS_RANGE_MULT: f32 = 2.0;
pub const CONDUCTING_BOLTS_DAMAGE_MULT: f32 = 0.7;
pub const HIGH_VOLTAGE_DAMAGE_MULT: f32 = 1.8;
pub const HIGH_VOLTAGE_FALLOFF: f32 = 0.4;
pub const STATIC_CHARGE_SLOW: f32 = -0.2;
pub const STATIC_CHARGE_DURATION: f32 = 2.0;

// Tier 2
pub const FORKED_SPLIT_COUNT: usize = 3;
pub const OVERCHARGE_SPLIT_COUNT: usize = 1;
pub const OVERCHARGE_MAX_BOUNCES: u32 = 5;
pub const OVERCHARGE_FALLOFF: f32 = 1.0;
pub const MAGNETIC_PULL_SPEED: f32 = 200.0;
pub const MAGNETIC_PULL_DURATION: f32 = 0.5;

// Tier 3
pub const THUNDERSTORM_CAST_COUNT: u32 = 3;
pub const THUNDERSTORM_MANA_MULT: f32 = 4.0;
pub const CHAIN_REACTION_AOE_RADIUS: f32 = 40.0;
pub const CHAIN_REACTION_AOE_DAMAGE_MULT: f32 = 0.5;
pub const CHAIN_REACTION_BOUNCE_DIVISOR: u32 = 2;
pub const LIVING_LIGHTNING_MAX_BOUNCES: u32 = 100;
pub const LIVING_LIGHTNING_MANA_MULT: f32 = 2.0;

// ===== Style Helper Functions =====

/// Returns the arc width scaled by split depth and empowerment.
/// Deeper splits produce thinner arcs.
pub fn arc_width_at_depth(depth: u32, empowerment: f32) -> f32 {
    let width = ARC_WIDTH * empowerment * ARC_WIDTH_FALLOFF.powi(depth as i32);
    width.max(MIN_ARC_WIDTH)
}
