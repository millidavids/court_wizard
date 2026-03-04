//! Disintegrate spell constants.

use crate::game::units::DamageType;
use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Disintegrate.
pub const PRIMED_DISINTEGRATE: PrimedSpell = PrimedSpell {
    spell: Spell::Disintegrate,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Damage dealt per tick to entities in the beam.
pub const DAMAGE_PER_TICK: f32 = 5.0;

/// Type of damage dealt by disintegrate beams.
pub const DAMAGE_TYPE: DamageType = DamageType::Fire;

/// Time between damage ticks (in seconds).
pub const DAMAGE_INTERVAL: f32 = 0.1;

/// Width of the beam for both collision detection and visual rendering.
pub const BEAM_WIDTH: f32 = 16.0;

/// Mana cost per second while channeling.
pub const MANA_COST_PER_SECOND: f32 = 20.0;

/// Beam length (extends through the battlefield).
pub const BEAM_LENGTH: f32 = 5000.0;

/// Height offset from spell origin where beam originates.
pub const BEAM_ORIGIN_HEIGHT_OFFSET: f32 = 0.0;

/// Cast time before beam activates (in seconds).
pub const CAST_TIME: f32 = 1.5;

/// Time for beam to grow to full length (in seconds).
pub const BEAM_GROWTH_TIME: f32 = 0.2;

/// Extra visual length past the beam tip on the ground surface.
pub(super) const BEAM_VISUAL_OVERSHOOT: f32 = 300.0;

// ── Core beam pulse ──────────────────────────────────────────────────
/// Oscillations per second for beam width pulsing.
pub(super) const BEAM_PULSE_FREQUENCY: f32 = 12.0;
/// Fraction of width variation (15%).
pub(super) const BEAM_PULSE_AMPLITUDE: f32 = 0.15;

// ── Outer glow layer ─────────────────────────────────────────────────
/// Glow cylinder is this many times wider than the core beam.
pub(super) const GLOW_WIDTH_MULTIPLIER: f32 = 2.5;
/// Oscillations per second for glow pulsing.
pub(super) const GLOW_PULSE_FREQUENCY: f32 = 8.0;
/// Fraction of glow width variation.
pub(super) const GLOW_PULSE_AMPLITUDE: f32 = 0.2;
/// First incommensurate frequency for shimmer jitter.
pub(super) const SHIMMER_FREQ_A: f32 = 47.0;
/// Second incommensurate frequency for shimmer jitter.
pub(super) const SHIMMER_FREQ_B: f32 = 31.0;
/// Amplitude of shimmer jitter.
pub(super) const SHIMMER_AMPLITUDE: f32 = 0.05;

// ── Origin flare ─────────────────────────────────────────────────────
/// Base radius of the flare triangle at beam origin.
pub(super) const FLARE_RADIUS: f32 = 32.0;
/// Oscillations per second for flare pulsing.
pub(super) const FLARE_PULSE_FREQUENCY: f32 = 15.0;
/// Fraction of flare size variation.
pub(super) const FLARE_PULSE_AMPLITUDE: f32 = 0.3;

// ── Impact particles ─────────────────────────────────────────────────
/// Time between particle batch spawns (seconds).
pub(super) const PARTICLE_SPAWN_INTERVAL: f32 = 0.04;
/// Number of particles per spawn batch.
pub(super) const PARTICLE_COUNT_PER_SPAWN: usize = 5;
/// How long each particle lives (seconds).
pub(super) const PARTICLE_LIFETIME: f32 = 0.3;
/// Base size of each particle.
pub(super) const PARTICLE_SIZE: f32 = 10.0;
/// Speed of particles flying outward from impact point.
pub(super) const PARTICLE_SPEED: f32 = 150.0;

// ── Smoke wisps ──────────────────────────────────────────────────────
/// Time between smoke wisp batch spawns (seconds).
pub(super) const SMOKE_SPAWN_INTERVAL: f32 = 0.08;
/// Number of smoke wisps per spawn batch per beam.
pub(super) const SMOKE_COUNT_PER_SPAWN: usize = 2;
/// How long each smoke wisp lives (seconds).
pub(super) const SMOKE_LIFETIME: f32 = 1.0;
/// Base size of each smoke triangle.
pub(super) const SMOKE_SIZE: f32 = 3.0;
/// Upward drift speed of smoke wisps.
pub(super) const SMOKE_RISE_SPEED: f32 = 60.0;
/// Lateral spread speed of smoke wisps.
pub(super) const SMOKE_SPREAD_SPEED: f32 = 30.0;

// ── Color cycling ────────────────────────────────────────────────────
/// Speed of the orange -> yellow -> white -> yellow -> orange cycle.
pub(super) const COLOR_CYCLE_SPEED: f32 = 4.0;

// ── Talent: Focused Lens (T1-0) ─────────────────────────────────────
/// Beam width multiplier for Focused Lens.
pub(super) const FOCUSED_LENS_WIDTH_MULT: f32 = 0.6;
/// Damage multiplier for Focused Lens.
pub(super) const FOCUSED_LENS_DAMAGE_MULT: f32 = 1.3;

// ── Talent: Unfocused Beam (T1-1) ───────────────────────────────────
/// Width multiplier for Unfocused Beam.
pub(super) const UNFOCUSED_BEAM_WIDTH_MULT: f32 = 2.0;
/// Damage multiplier for Unfocused Beam.
pub(super) const UNFOCUSED_BEAM_DAMAGE_MULT: f32 = 0.3;

// ── Talent: Efficient Channeling (T1-2) ─────────────────────────────
/// Mana cost multiplier for Efficient Channeling.
pub(super) const EFFICIENT_CHANNELING_MANA_MULT: f32 = 0.7;

// ── Talent: Forked Beam (T2-0) ──────────────────────────────────────
/// Half-angle of the fan spread (radians, ~7.5 degrees).
pub(super) const FORKED_FAN_HALF_ANGLE: f32 = 0.13;
/// Damage multiplier per beam for Forked Beam.
pub(super) const FORKED_DAMAGE_MULT: f32 = 0.5;

// ── Talent: Escalating Intensity (T2-1) ─────────────────────────────
/// Starting damage multiplier at channel begin.
pub(super) const ESCALATING_START_MULT: f32 = 0.5;
/// Maximum damage multiplier at full ramp.
pub(super) const ESCALATING_MAX_MULT: f32 = 2.0;
/// Time to ramp from start to max (seconds).
pub(super) const ESCALATING_RAMP_TIME: f32 = 4.0;

// ── Talent: Sweeping Destruction (T2-2) ──────────────────────────────
/// Angular speed of the sweep (radians/second).
pub(super) const SWEEP_SPEED: f32 = 0.8;
/// Half-arc of the sweep (radians, ~22.5 degrees).
pub(super) const SWEEP_HALF_ARC: f32 = 0.39;
/// Damage multiplier for Sweeping Destruction (+100% damage).
pub(super) const SWEEPING_DAMAGE_MULT: f32 = 2.0;

// ── Talent: Annihilation Beam (T3-0) ─────────────────────────────────
/// Width multiplier for Annihilation Beam.
pub(super) const ANNIHILATION_WIDTH_MULT: f32 = 3.0;
/// Damage multiplier for Annihilation Beam.
pub(super) const ANNIHILATION_DAMAGE_MULT: f32 = 2.0;
/// Mana cost multiplier for Annihilation Beam.
pub(super) const ANNIHILATION_MANA_MULT: f32 = 2.0;
/// Height from which the Annihilation Beam descends.
pub(super) const ANNIHILATION_SKY_HEIGHT: f32 = 2000.0;
/// Lateral spread between forked sky beams (units from center to outer beam).
pub(super) const ANNIHILATION_FORKED_SPREAD: f32 = 180.0;
/// Sweep radius for annihilation beam sweeping (units per radian of sweep angle).
pub(super) const ANNIHILATION_SWEEP_RADIUS: f32 = 500.0;

// ── Talent: Searing Finale (T3-1) ───────────────────────────────────
/// Damage multiplier for the detonation (relative to beam DPS).
pub(super) const SEARING_FINALE_DAMAGE_MULT: f32 = 3.0;
/// Duration of the detonation visual (seconds).
pub(super) const SEARING_FINALE_DURATION: f32 = 0.5;
/// Width multiplier for the detonation.
pub(super) const SEARING_FINALE_WIDTH_MULT: f32 = 2.0;

// ── Talent: Beam Fireballs (T3-2) ───────────────────────────────────
/// Time between mini-fireball spawns (seconds).
pub(super) const MINI_FIREBALL_INTERVAL: f32 = 1.5;
/// Fraction of fireball TOTAL_DAMAGE for each mini-fireball.
pub(super) const MINI_FIREBALL_DAMAGE_FRACTION: f32 = 0.1;
