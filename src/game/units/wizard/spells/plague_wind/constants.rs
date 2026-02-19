use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_PLAGUE_WIND: PrimedSpell = PrimedSpell {
    spell: Spell::PlagueWind,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 35.0;
pub const CLOUD_RADIUS: f32 = 100.0;
pub const CLOUD_DURATION: f32 = 12.0;
pub const CLOUD_SPEED: f32 = 40.0;
pub const DAMAGE_PER_TICK: f32 = 5.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const FADE_DURATION: f32 = 2.0;

pub const CIRCLE_COLOR: bevy::color::Color = bevy::color::Color::srgba(0.3, 0.8, 0.1, 0.3);
pub const CIRCLE_Y_POSITION: f32 = 0.5;
pub const CLOUD_COLOR: bevy::color::Color = bevy::color::Color::srgba(0.2, 0.6, 0.1, 0.4);
