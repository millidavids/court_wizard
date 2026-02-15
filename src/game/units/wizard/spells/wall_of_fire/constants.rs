use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_WALL_OF_FIRE: PrimedSpell = PrimedSpell {
    spell: Spell::WallOfFire,
    cast_time: 0.0, // Instant — placed on release like Wall of Stone
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const MANA_COST: f32 = 35.0;
pub const MIN_WALL_LENGTH: f32 = 20.0;
pub const MAX_WALL_LENGTH: f32 = 600.0;
pub const WALL_WIDTH: f32 = 60.0;
pub const FIRE_DURATION: f32 = 12.0;
pub const DAMAGE_PER_TICK: f32 = 5.0;
pub const TICK_INTERVAL: f32 = 0.25;
pub const FADE_DURATION: f32 = 1.0;
pub const PREVIEW_COLOR: Color = Color::srgba(1.0, 0.4, 0.0, 0.3);
