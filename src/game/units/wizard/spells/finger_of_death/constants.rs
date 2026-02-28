use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_FINGER_OF_DEATH: PrimedSpell = PrimedSpell {
    spell: Spell::FingerOfDeath,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// Casting
pub const CAST_TIME: f32 = 2.0;
pub const BEAM_ORIGIN_HEIGHT_OFFSET: f32 = 0.0;

// Mana requirement - must have at least 50% mana
pub const MANA_REQUIREMENT_PERCENT: f32 = 0.5;

// Damage
pub const DAMAGE: f32 = 1000.0;

// Beam properties
pub const BEAM_LENGTH: f32 = 5000.0;
pub const BEAM_WIDTH: f32 = 30.0; // 30 pixels wide
pub const BEAM_WIDTH_FIRED: f32 = 30.0; // Same width after firing

// Visuals
pub const BEAM_COLOR_CASTING: Color = Color::srgb(0.6, 0.0, 0.8); // Dark purple
pub const BEAM_COLOR_FIRED: Color = Color::srgb(0.8, 0.0, 1.0); // Bright purple
pub const ALPHA_CASTING: f32 = 0.5; // 50% opacity during cast

// Timing
pub const POST_FIRE_DURATION: f32 = 0.3; // Beam persists for 0.3s after firing, fading out

// Necrotic vein particles
pub const VEIN_COUNT: usize = 5;
pub const VEIN_LIFETIME: f32 = 1.5;
pub const VEIN_SIZE: f32 = 10.0;
pub const VEIN_SPEED: f32 = 80.0;
pub const VEIN_WANDER_FREQUENCY: f32 = 6.0;
pub const VEIN_WANDER_AMPLITUDE: f32 = 60.0;
pub const VEIN_Y_POSITION: f32 = 2.0;

// Glow aura
pub const GLOW_WIDTH_MULTIPLIER: f32 = 2.0;
pub const GLOW_PULSE_FREQUENCY: f32 = 8.0;
pub const GLOW_PULSE_AMPLITUDE: f32 = 0.15;

// Necrotic pulse ring
pub const PULSE_LIFETIME: f32 = 0.5;
pub const PULSE_MAX_RADIUS: f32 = 200.0;
pub const PULSE_Y_POSITION: f32 = 2.0;
