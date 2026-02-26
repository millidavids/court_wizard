use bevy::prelude::*;

use crate::game::units::DamageType;
use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_CHAIN_LIGHTNING: PrimedSpell = PrimedSpell {
    spell: Spell::ChainLightning,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// Casting
pub const CAST_TIME: f32 = 0.4;
pub const MANA_COST: f32 = 15.0;
pub const SPAWN_HEIGHT_OFFSET: f32 = 0.0;

// Damage
pub const INITIAL_DAMAGE: f32 = 20.0;
pub const DAMAGE_TYPE: DamageType = DamageType::Electric;
pub const DAMAGE_FALLOFF: f32 = 0.6;
pub const MAX_BOUNCES: u32 = 8;

// Splitting
pub const SPLIT_COUNT: usize = 2;

// Targeting
pub const TARGETING_RADIUS: f32 = 50.0; // Cursor proximity to enemy
pub const BOUNCE_RANGE: f32 = 100.0; // Max distance between targets

// Timing
pub const BOUNCE_DELAY: f32 = 0.05; // Time between bounces
pub const ARC_LIFETIME: f32 = 0.3; // Arc visual persistence

// Visuals
pub const ARC_SEGMENTS: u32 = 8; // Number of segments per arc (for curved path)
pub const ARC_HEIGHT_FACTOR: f32 = 0.15; // Base peak height as fraction of horizontal distance
pub const ARC_HEIGHT_GROWTH: f32 = 0.12; // Additional height factor per depth level
pub const ARC_WIDTH: f32 = 8.0;
pub const ARC_COLOR: Color = Color::srgb(0.7, 0.85, 1.0); // Electric blue
pub const ARC_WIDTH_FALLOFF: f32 = 0.8; // Width multiplier per depth level
pub const MIN_ARC_WIDTH: f32 = 2.0;
pub const ARC_BRIGHTNESS_FALLOFF: f32 = 0.92; // Brightness multiplier per depth level
pub const MIN_ARC_BRIGHTNESS: f32 = 0.4;
