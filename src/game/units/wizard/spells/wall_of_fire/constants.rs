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
pub const FIRE_DURATION: f32 = 20.0;
pub const DAMAGE_PER_TICK: f32 = 1.0;
pub const TICK_INTERVAL: f32 = 0.25;
pub const FADE_DURATION: f32 = 1.0;
pub const PREVIEW_COLOR: Color = Color::srgba(1.0, 0.4, 0.0, 0.3);

// Tier 1 talent constants
pub(super) const INFERNAL_INTENSITY_DAMAGE_MULT: f32 = 2.0;
pub(super) const FIREBREAK_WIDTH_MULT: f32 = 1.8;
pub(super) const FIREBREAK_DURATION_MULT: f32 = 1.25;
pub(super) const FLASH_FIRE_MAX_LENGTH_MULT: f32 = 1.5;
pub(super) const FLASH_FIRE_DAMAGE_MULT: f32 = 1.5;
pub(super) const FLASH_FIRE_DURATION_MULT: f32 = 0.6;

// Tier 2 talent constants
pub(super) const SEARING_HEAT_HEALING_REDUCTION: f32 = 0.5;
pub(super) const SCORCHED_EARTH_DURATION: f32 = 8.0;
pub(super) const SCORCHED_EARTH_SLOW: f32 = -0.3;
pub(super) const SCORCHED_EARTH_SLOW_DURATION: f32 = 1.0;
pub(super) const SCORCHED_EARTH_TICK_INTERVAL: f32 = 0.5;
pub(super) const SPREADING_FLAMES_DURATION: f32 = 3.0;
pub(super) const SPREADING_FLAMES_DAMAGE_FRACTION: f32 = 0.5;

// Tier 3 talent constants
pub(super) const FIRESTORM_EXPLOSION_RADIUS: f32 = 50.0;
pub(super) const FIRESTORM_EXPLOSION_DAMAGE: f32 = 15.0;
pub(super) const FIRESTORM_EXPLOSION_DURATION: f32 = 0.3;
pub(super) const TWIN_WALLS_DAMAGE_MULT: f32 = 0.6;
pub(super) const CONSUMING_INFERNO_RAMP_PER_SECOND: f32 = 0.15;
pub(super) const CONSUMING_INFERNO_MAX_RAMP: f32 = 3.0;
