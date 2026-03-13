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
pub const CIRCLE_Y_POSITION: f32 = 2.0;
pub const FADE_DURATION: f32 = 2.0;
