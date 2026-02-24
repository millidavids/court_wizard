use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_SLEEP: PrimedSpell = PrimedSpell {
    spell: Spell::Sleep,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 2.5;
pub const MANA_COST: f32 = 35.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const SLEEP_DURATION: f32 = 8.0;
pub const BONUS_DAMAGE_MULTIPLIER: f32 = 2.0;
pub const CIRCLE_Y_POSITION: f32 = 1.0;
