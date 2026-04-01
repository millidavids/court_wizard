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
pub const COOLDOWN: f32 = 0.0; // No cooldown by default (blocked by mouse release)

// Mana requirement - must have at least 50% mana
pub const MANA_REQUIREMENT_PERCENT: f32 = 0.5;

// Damage
pub const DAMAGE: f32 = 500.0;

// Beam properties
pub const BEAM_LENGTH: f32 = 5000.0;
pub const BEAM_WIDTH: f32 = 30.0; // 30 pixels wide
pub const BEAM_WIDTH_FIRED: f32 = 30.0; // Same width after firing

// Timing
pub const POST_FIRE_DURATION: f32 = 0.3; // Beam persists for 0.3s after firing, fading out

// Beam triangle visuals (disintegrate-style)
pub(super) const BEAM_PULSE_FREQUENCY: f32 = 10.0;
pub(super) const BEAM_PULSE_AMPLITUDE: f32 = 0.15;
pub(super) const COLOR_CYCLE_SPEED: f32 = 3.0;
pub(super) const BEAM_VISUAL_OVERSHOOT: f32 = 50.0;

// Glow shimmer
pub(super) const SHIMMER_FREQ_A: f32 = 41.0;
pub(super) const SHIMMER_FREQ_B: f32 = 29.0;
pub(super) const SHIMMER_AMPLITUDE: f32 = 0.05;

// Origin flare
pub(super) const FLARE_RADIUS: f32 = 28.0;
pub(super) const FLARE_PULSE_FREQUENCY: f32 = 12.0;
pub(super) const FLARE_PULSE_AMPLITUDE: f32 = 0.3;

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

// === Talent constants ===

// Tier 1
pub(super) const DEATHS_REACH_WIDTH_MULT: f32 = 1.5;
pub(super) const SOUL_HARVEST_MANA_REFUND: f32 = 0.15;
pub(super) const QUICK_DRAW_CAST_MULT: f32 = 0.6;

// Tier 2
pub(super) const DEATH_SENTENCE_MANA_THRESHOLD: f32 = 0.3;
pub(super) const DEATH_SENTENCE_DAMAGE: f32 = 700.0;
pub(super) const DEATH_SENTENCE_COOLDOWN_MULT: f32 = 0.5;
pub(super) const SIPHON_LIFE_HEAL_PERCENT: f32 = 0.5;

// Tier 3
pub(super) const REAPERS_SCYTHE_SWEEP_DURATION: f32 = 1.0;
pub(super) const REAPERS_SCYTHE_DAMAGE_MULT: f32 = 0.3;
pub(super) const REAPERS_SCYTHE_ARC_DEGREES: f32 = 120.0;
pub(super) const NECROTIC_EXPLOSION_DAMAGE_PERCENT: f32 = 0.2;
pub(super) const NECROTIC_EXPLOSION_RADIUS: f32 = 100.0;
pub(super) const DEATHMARK_DURATION: f32 = 5.0;
pub(super) const DEATHMARK_CHAIN_DAMAGE_PERCENT: f32 = 0.1;

// Necrotic explosion visuals
pub(super) const NECROTIC_EXPLOSION_PULSE_LIFETIME: f32 = 0.4;
pub(super) const NECROTIC_EXPLOSION_COLOR: Color = Color::srgb(0.5, 0.0, 0.7);
