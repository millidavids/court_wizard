use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_PHANTASMAL_FORCE: PrimedSpell = PrimedSpell {
    spell: Spell::PhantasmalForce,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.0;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 50.0;
pub const DECOY_COUNT: u32 = 3;
pub const DECOY_HP: f32 = 1.0;
pub const DECOY_DURATION: f32 = 12.0;
pub const DECOY_SPREAD: f32 = 30.0;
pub const CIRCLE_Y_POSITION: f32 = 1.0;
