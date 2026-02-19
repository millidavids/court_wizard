use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_BANISHMENT: PrimedSpell = PrimedSpell {
    spell: Spell::Banishment,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.0;
pub const MANA_COST: f32 = 40.0;
pub const BANISH_DURATION: f32 = 8.0;
pub const TARGET_SEARCH_RADIUS: f32 = 80.0;
