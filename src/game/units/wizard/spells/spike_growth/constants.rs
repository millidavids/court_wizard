use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_SPIKE_GROWTH: PrimedSpell = PrimedSpell {
    spell: Spell::SpikeGrowth,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 2.5;
pub const MANA_COST: f32 = 30.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const DAMAGE_PER_TICK: f32 = 3.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const ZONE_DURATION: f32 = 15.0;
pub const SLOW_MODIFIER: f32 = -0.3;
pub const SLOW_DURATION: f32 = 2.0;
pub const CIRCLE_Y_POSITION: f32 = 1.0;
pub const CIRCLE_COLOR: Color = Color::srgba(0.2, 0.45, 0.1, 0.3);
pub const ZONE_COLOR: Color = Color::srgba(0.15, 0.4, 0.05, 0.4);
pub const FADE_DURATION: f32 = 2.0;
