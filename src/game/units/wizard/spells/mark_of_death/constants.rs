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
