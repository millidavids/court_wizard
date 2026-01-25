use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_CHAIN_LIGHTNING: PrimedSpell = PrimedSpell {
    spell: Spell::ChainLightning,
    cast_time: CAST_TIME,
};

// Casting
pub const CAST_TIME: f32 = 0.8;
pub const MANA_COST: f32 = 25.0;
pub const SPAWN_HEIGHT_OFFSET: f32 = 100.0;

// Damage
pub const INITIAL_DAMAGE: f32 = 40.0;
pub const DAMAGE_FALLOFF: f32 = 0.7;
pub const MAX_BOUNCES: u32 = 4;

// Targeting
pub const TARGETING_RADIUS: f32 = 50.0; // Cursor proximity to enemy
pub const BOUNCE_RANGE: f32 = 150.0; // Max distance between targets

// Timing
pub const BOUNCE_DELAY: f32 = 0.05; // Time between bounces
pub const ARC_LIFETIME: f32 = 0.3; // Arc visual persistence

// Visuals
pub const ARC_WIDTH: f32 = 8.0;
pub const ARC_COLOR: Color = Color::srgb(0.7, 0.85, 1.0); // Electric blue
