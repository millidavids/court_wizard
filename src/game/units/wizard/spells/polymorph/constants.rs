use crate::game::units::wizard::components::{PrimedSpell, Spell};
use bevy::prelude::*;

pub const PRIMED_POLYMORPH: PrimedSpell = PrimedSpell {
    spell: Spell::Polymorph,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 0.8;
pub const MANA_COST: f32 = 35.0;
pub const POLYMORPH_DURATION: f32 = 10.0;
pub const SHEEP_HP: f32 = 20.0;
pub const TARGET_SEARCH_RADIUS: f32 = 80.0;
pub const SHEEP_COLOR: Color = Color::srgba(0.9, 0.9, 0.85, 1.0);
