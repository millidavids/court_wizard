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
pub const CIRCLE_Y_POSITION: f32 = 1.0;
pub const FADE_DURATION: f32 = 2.0;
