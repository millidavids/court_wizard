use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_HASTE: PrimedSpell = PrimedSpell {
    spell: Spell::Haste,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 20.0;
pub const CIRCLE_RADIUS: f32 = 150.0;
pub const HASTE_MODIFIER: f32 = 0.5;
pub const HASTE_DURATION: f32 = 10.0;

// ===== Talent Constants =====

// Tier 1
/// Alacrity: speed bonus multiplier (+40%).
pub const ALACRITY_SPEED_MULT: f32 = 1.4;
/// Extended Rush: duration multiplier (+50%).
pub const EXTENDED_RUSH_DURATION_MULT: f32 = 1.5;
/// Quick Cast: cast time multiplier (-50%).
pub const QUICK_CAST_CAST_TIME_MULT: f32 = 0.5;

// Tier 2
/// Adrenaline Surge: attack speed bonus (+20%).
pub const ADRENALINE_SURGE_ATTACK_SPEED: f32 = 0.2;
/// Momentum: damage bonus after buff expires (+25%).
pub const MOMENTUM_DAMAGE_MULT: f32 = 0.25;
/// Momentum: duration of post-buff damage bonus (seconds).
pub const MOMENTUM_DURATION: f32 = 2.0;

// Tier 3
/// Time Warp: bonus multiplier for speed and attack speed (2x).
pub const TIME_WARP_BONUS_MULT: f32 = 2.0;
/// Time Warp: duration multiplier (halved).
pub const TIME_WARP_DURATION_MULT: f32 = 0.5;
/// Slow Zone: slow percentage applied to enemies (-30% speed).
pub const SLOW_ZONE_SLOW_AMOUNT: f32 = -0.3;
/// Slow Zone: duration the slow field persists (seconds).
pub const SLOW_ZONE_DURATION: f32 = 8.0;
/// Slow Zone: radius of the slow field (same as spell circle).
pub const SLOW_ZONE_RADIUS: f32 = CIRCLE_RADIUS;
/// Chain Haste: maximum number of hops.
pub const CHAIN_HASTE_MAX_HOPS: u32 = 4;
/// Chain Haste: effectiveness retained per hop (80%).
pub const CHAIN_HASTE_FALLOFF: f32 = 0.8;
/// Chain Haste: search radius for next un-hasted ally.
pub const CHAIN_HASTE_RADIUS: f32 = 200.0;
