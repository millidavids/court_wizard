use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_MARK_OF_DEATH: PrimedSpell = PrimedSpell {
    spell: Spell::MarkOfDeath,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 0.5;
pub const MANA_COST: f32 = 20.0;
pub const DAMAGE_AMPLIFICATION: f32 = 0.5;
pub const MARK_DURATION: f32 = 8.0;
pub const TARGET_SEARCH_RADIUS: f32 = 80.0;

// === Talent constants ===

// Tier 1
pub const DEEP_MARK_AMPLIFICATION: f32 = 0.75;
pub const LINGERING_CURSE_DURATION: f32 = 12.0;
pub const SWIFT_HEX_REFUND_PERCENT: f32 = 0.5;

// Tier 2
pub const SPREADING_BLIGHT_DURATION_PERCENT: f32 = 0.5;
pub const EXECUTIONER_HP_THRESHOLD: f32 = 0.3;
pub const EXECUTIONER_BURST_DAMAGE: f32 = 50.0;

// Tier 3
pub const MASS_MARKING_RADIUS: f32 = 60.0;
pub const MASS_MARKING_AMPLIFICATION: f32 = 0.35;
pub const DEATHS_LEDGER_DAMAGE_PER_MAX_HP: f32 = 0.3;
pub const DEATHS_LEDGER_RADIUS: f32 = 80.0;
pub const DEATHS_LEDGER_PULSE_LIFETIME: f32 = 0.8;
pub const DOOM_AMP_PER_SECOND: f32 = 0.10;

// Visual indicator
pub const MARK_INDICATOR_RADIUS: f32 = 8.0;
pub const MARK_INDICATOR_Y_OFFSET: f32 = 110.0;
pub const MARK_INDICATOR_PULSE_SPEED: f32 = 3.0;
pub const MARK_INDICATOR_PULSE_AMPLITUDE: f32 = 0.15;
