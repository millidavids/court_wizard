use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_ENTANGLE: PrimedSpell = PrimedSpell {
    spell: Spell::Entangle,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 2.0;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const ROOT_DURATION: f32 = 5.0;
pub const CIRCLE_Y_POSITION: f32 = 1.0;
pub const CIRCLE_COLOR: Color = Color::srgba(0.1, 0.7, 0.2, 0.3);
pub const GROUND_EFFECT_COLOR: Color = Color::srgba(0.1, 0.6, 0.15, 0.35);
