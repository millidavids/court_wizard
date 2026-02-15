use bevy::prelude::*;

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
pub const CIRCLE_Y_POSITION: f32 = 1.0;
pub const CIRCLE_COLOR: Color = Color::srgba(1.0, 0.85, 0.0, 0.3);
