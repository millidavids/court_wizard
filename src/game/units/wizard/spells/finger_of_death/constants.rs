use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_FINGER_OF_DEATH: PrimedSpell = PrimedSpell {
    spell: Spell::FingerOfDeath,
    cast_time: CAST_TIME,
};

// Casting
pub const CAST_TIME: f32 = 2.0;
pub const BEAM_ORIGIN_HEIGHT_OFFSET: f32 = 100.0;

// Mana requirement - must have full mana (100%)
pub const MANA_REQUIREMENT_PERCENT: f32 = 1.0;

// Damage
pub const DAMAGE: f32 = 1000.0;

// Beam properties
pub const BEAM_LENGTH: f32 = 5000.0;
pub const BEAM_WIDTH: f32 = 10.0; // 10 pixels wide
pub const BEAM_WIDTH_FIRED: f32 = 10.0; // Same width after firing

// Visuals
pub const BEAM_COLOR_CASTING: Color = Color::srgb(0.6, 0.0, 0.8); // Dark purple
pub const BEAM_COLOR_FIRED: Color = Color::srgb(0.8, 0.0, 1.0); // Bright purple
pub const ALPHA_CASTING: f32 = 0.5; // 50% opacity during cast

// Timing
pub const POST_FIRE_DURATION: f32 = 0.3; // Beam persists for 0.3s after firing, fading out
