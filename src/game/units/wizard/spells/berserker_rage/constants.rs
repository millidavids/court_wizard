use crate::game::units::wizard::components::{PrimedSpell, Spell};
use bevy::prelude::*;

pub const PRIMED_BERSERKER_RAGE: PrimedSpell = PrimedSpell {
    spell: Spell::BerserkerRage,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.0;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 150.0;
pub const DAMAGE_BONUS: f32 = 0.8;
pub const DAMAGE_VULNERABILITY: f32 = 0.5;
pub const BUFF_DURATION: f32 = 8.0;
pub const CIRCLE_Y_POSITION: f32 = 1.0;
pub const CIRCLE_COLOR: Color = Color::srgba(0.9, 0.15, 0.1, 0.3);
