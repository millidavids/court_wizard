use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_FOG_CLOUD: PrimedSpell = PrimedSpell {
    spell: Spell::FogCloud,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.0;
pub const MANA_COST: f32 = 30.0;
pub const CIRCLE_RADIUS: f32 = 200.0;
pub const EVASION_CHANCE: f32 = 0.4;
pub const EVASION_REFRESH_DURATION: f32 = 1.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const ZONE_DURATION: f32 = 12.0;
pub const FADE_DURATION: f32 = 2.0;

// Talent constants
// Tier 1
pub const DENSE_FOG_EVASION: f32 = 0.55;
pub const EXPANDING_MISTS_RADIUS_MULT: f32 = 1.4;
pub const CLINGING_HAZE_LINGER: f32 = 2.0;
// Tier 2
pub const BLINDING_MIST_RANGE_MULT: f32 = 0.5;
pub const BLINDING_MIST_DEBUFF_DURATION: f32 = 1.0;
pub const DISORIENTING_VAPORS_CHANCE: f32 = 0.2;
// Tier 3
pub const PHANTOM_SPAWN_INTERVAL: f32 = 3.0;
pub const PHANTOM_MAX_TOTAL: usize = 3;
pub const PHANTOM_HITBOX_RADIUS: f32 = 8.0;
pub const PHANTOM_HITBOX_HEIGHT: f32 = 16.0;
pub const CHOKING_FOG_DPS: f32 = 3.0;
pub const CHOKING_FOG_TICK_INTERVAL: f32 = 0.5;
pub const ROLLING_FOG_SPEED: f32 = 30.0;
