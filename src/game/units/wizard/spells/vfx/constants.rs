//! Shared visual effect constants.

use bevy::prelude::Quat;

/// Rotation to make a circle mesh (XY plane) lie flat facing upward (XZ plane).
pub(crate) const UPWARD_ROTATION: Quat = Quat::from_xyzw(
    -std::f32::consts::FRAC_1_SQRT_2,
    0.0,
    0.0,
    std::f32::consts::FRAC_1_SQRT_2,
);

// ── Projectile glow ──────────────────────────────────────────────────
/// Glow sphere is this many times wider than the projectile.
pub(crate) const GLOW_SIZE_MULTIPLIER: f32 = 2.0;
/// Oscillations per second for glow pulsing.
pub(crate) const GLOW_PULSE_FREQUENCY: f32 = 10.0;
/// Fraction of glow size variation.
pub(crate) const GLOW_PULSE_AMPLITUDE: f32 = 0.15;

// ── Fire smoke trail ─────────────────────────────────────────────────
/// Time between smoke wisp batch spawns (seconds).
pub(crate) const SMOKE_SPAWN_INTERVAL: f32 = 0.08;
/// Number of smoke wisps per spawn batch per source.
pub(crate) const SMOKE_COUNT_PER_SPAWN: usize = 2;
/// How long each smoke wisp lives (seconds).
pub(crate) const SMOKE_LIFETIME: f32 = 0.8;
/// Base size of each smoke triangle.
pub(crate) const SMOKE_SIZE: f32 = 3.0;
/// Upward drift speed of smoke wisps.
pub(crate) const SMOKE_RISE_SPEED: f32 = 40.0;
/// Lateral spread speed of smoke wisps.
pub(crate) const SMOKE_SPREAD_SPEED: f32 = 20.0;

// ── Explosion smoke ──────────────────────────────────────────────────
/// Smoke spawned per explosion.
pub(crate) const EXPLOSION_SMOKE_COUNT: usize = 20;
/// Lifetime of explosion smoke wisps (seconds).
pub(crate) const EXPLOSION_SMOKE_LIFETIME: f32 = 1.4;
/// Size of explosion smoke triangles.
pub(crate) const EXPLOSION_SMOKE_SIZE: f32 = 8.0;
/// Rise speed of explosion smoke.
pub(crate) const EXPLOSION_SMOKE_RISE_SPEED: f32 = 50.0;
/// Lateral spread of explosion smoke.
pub(crate) const EXPLOSION_SMOKE_SPREAD: f32 = 90.0;

// ── Impact sparks ────────────────────────────────────────────────────
/// Number of sparks spawned per explosion.
pub(crate) const SPARK_COUNT: usize = 32;
/// How long each spark lives (seconds).
pub(crate) const SPARK_LIFETIME: f32 = 0.5;
/// Base size of each spark triangle.
pub(crate) const SPARK_SIZE: f32 = 2.5;
/// Speed of sparks flying outward from impact.
pub(crate) const SPARK_SPEED: f32 = 180.0;
/// Minimum spark elevation (fraction, 0=horizontal 1=vertical).
pub(crate) const SPARK_ELEVATION_MIN: f32 = 0.05;
/// Maximum spark elevation (fraction).
pub(crate) const SPARK_ELEVATION_MAX: f32 = 0.45;

// ── Explosion heat shimmer ───────────────────────────────────────────
/// Number of heat shimmer billboards spawned per explosion.
pub(crate) const EXPLOSION_SHIMMER_COUNT: usize = 4;

// ── Explosion dark smoke (lingering aftermath) ───────────────────────
/// Number of dark smoke puffs spawned per explosion.
pub(crate) const DARK_SMOKE_COUNT: usize = 5;
/// Lifetime of dark smoke puffs (seconds).
pub(crate) const DARK_SMOKE_LIFETIME: f32 = 1.2;
/// Base size of dark smoke puffs.
pub(crate) const DARK_SMOKE_SIZE: f32 = 15.0;
/// Gentle upward drift speed of dark smoke.
pub(crate) const DARK_SMOKE_RISE_SPEED: f32 = 10.0;
/// Lateral spread speed of dark smoke.
pub(crate) const DARK_SMOKE_SPREAD_SPEED: f32 = 12.0;

// ── Magic missile glow ─────────────────────────────────────────────
/// Glow is this many times wider than the missile.
pub(crate) const MISSILE_GLOW_SIZE_MULTIPLIER: f32 = 1.5;
/// Oscillations per second for missile glow pulsing.
pub(crate) const MISSILE_GLOW_PULSE_FREQUENCY: f32 = 8.0;
/// Fraction of glow size variation.
pub(crate) const MISSILE_GLOW_PULSE_AMPLITUDE: f32 = 0.2;

// ── Magic missile sparkle trail ────────────────────────────────────
/// Time between sparkle batch spawns (seconds).
pub(crate) const SPARKLE_SPAWN_INTERVAL: f32 = 0.03;
/// Number of sparkles per spawn batch.
pub(crate) const SPARKLE_COUNT_PER_SPAWN: usize = 2;
/// How long each sparkle lives (seconds).
pub(crate) const SPARKLE_LIFETIME: f32 = 0.5;
/// Base size of each sparkle particle.
pub(crate) const SPARKLE_SIZE: f32 = 2.0;
/// How quickly sparkles decelerate (fraction of velocity lost per second).
pub(crate) const SPARKLE_DRAG: f32 = 3.0;
/// Random spread speed added to sparkles on spawn.
pub(crate) const SPARKLE_SPREAD_SPEED: f32 = 30.0;

// ── Surface fire particles (wall of fire, grease fire, ground fire pools) ──
/// Size of smoke wisps rising off surface fires (2× trail size).
pub(crate) const SURFACE_SMOKE_SIZE: f32 = 6.0;
/// Number of smoke wisps per spawn batch for surface fires.
pub(crate) const SURFACE_SMOKE_COUNT: usize = 2;
/// Number of shimmer wisps per spawn batch for surface fires.
pub(crate) const SURFACE_SHIMMER_COUNT: usize = 2;
/// Size of shimmer billboards for surface fires (2× base size).
pub(crate) const SURFACE_SHIMMER_SIZE: f32 = 16.0;

// ── Plague smoke (poison cloud) ────────────────────────────────────
/// Time between plague smoke batch spawns (seconds).
pub(crate) const PLAGUE_SMOKE_SPAWN_INTERVAL: f32 = 0.06;
/// Number of smoke puffs per spawn batch.
pub(crate) const PLAGUE_SMOKE_COUNT_PER_SPAWN: usize = 3;
/// How long each plague smoke puff lives (seconds).
pub(crate) const PLAGUE_SMOKE_LIFETIME: f32 = 2.0;
/// Base size of each plague smoke particle.
pub(crate) const PLAGUE_SMOKE_SIZE: f32 = 25.0;
/// Upward drift speed of plague smoke.
pub(crate) const PLAGUE_SMOKE_RISE_SPEED: f32 = 12.0;
/// Lateral swirl speed of plague smoke puffs.
pub(crate) const PLAGUE_SMOKE_SWIRL_SPEED: f32 = 8.0;
/// Lateral sway frequency (Hz) for organic motion.
pub(crate) const PLAGUE_SMOKE_SWAY_FREQUENCY: f32 = 1.5;
/// Lateral sway amplitude (world units).
pub(crate) const PLAGUE_SMOKE_SWAY_AMPLITUDE: f32 = 8.0;

// ── Fog smoke (fog cloud) ──────────────────────────────────────────
/// Time between fog smoke batch spawns (seconds).
pub(crate) const FOG_SMOKE_SPAWN_INTERVAL: f32 = 0.04;
/// Number of fog smoke puffs per spawn batch.
pub(crate) const FOG_SMOKE_COUNT_PER_SPAWN: usize = 5;
/// How long each fog smoke puff lives (seconds).
pub(crate) const FOG_SMOKE_LIFETIME: f32 = 2.5;
/// Base size of each fog smoke particle.
pub(crate) const FOG_SMOKE_SIZE: f32 = 35.0;
/// Upward drift speed of fog smoke.
pub(crate) const FOG_SMOKE_RISE_SPEED: f32 = 8.0;
/// Lateral swirl speed of fog smoke puffs.
pub(crate) const FOG_SMOKE_SWIRL_SPEED: f32 = 5.0;

// ── Heat shimmer (fire haze) ────────────────────────────────────────
/// How long each shimmer wisp lives (seconds).
pub(crate) const SHIMMER_LIFETIME: f32 = 0.6;
/// Base size of each shimmer billboard.
pub(crate) const SHIMMER_SIZE: f32 = 8.0;
/// Gentle upward drift speed.
pub(crate) const SHIMMER_RISE_SPEED: f32 = 25.0;
/// Slight lateral wobble speed.
pub(crate) const SHIMMER_SPREAD_SPEED: f32 = 15.0;
/// Hz for lateral sway oscillation.
pub(crate) const SHIMMER_SWAY_FREQUENCY: f32 = 3.0;
/// Pixels of lateral sway.
pub(crate) const SHIMMER_SWAY_AMPLITUDE: f32 = 5.0;

// ── Cast flare (muzzle flash at wizard staff) ──────────────────────
/// How long the cast flare glow lasts (seconds).
pub(crate) const CAST_FLARE_LIFETIME: f32 = 0.35;
/// Base radius of the expanding glow ring.
pub(crate) const CAST_FLARE_SIZE: f32 = 12.0;
/// Number of sparks emitted by a cast flare.
pub(crate) const CAST_FLARE_SPARK_COUNT: usize = 10;
/// Speed of cast flare sparks.
pub(crate) const CAST_FLARE_SPARK_SPEED: f32 = 120.0;
/// Lifetime of cast flare sparks (seconds).
pub(crate) const CAST_FLARE_SPARK_LIFETIME: f32 = 0.3;
/// Size of cast flare spark particles.
pub(crate) const CAST_FLARE_SPARK_SIZE: f32 = 2.5;

// ── Floating motes (ambient zone particles) ─────────────────────────
/// How long each floating mote lives (seconds).
pub(crate) const MOTE_LIFETIME: f32 = 1.8;
/// Base size of each floating mote.
pub(crate) const MOTE_SIZE: f32 = 2.5;
/// Upward drift speed of motes.
pub(crate) const MOTE_RISE_SPEED: f32 = 15.0;
/// Lateral spread speed of motes.
pub(crate) const MOTE_SPREAD_SPEED: f32 = 10.0;
/// Lateral sway frequency (Hz) for motes.
pub(crate) const MOTE_SWAY_FREQUENCY: f32 = 2.0;
/// Lateral sway amplitude (world units) for motes.
pub(crate) const MOTE_SWAY_AMPLITUDE: f32 = 4.0;
/// Time between mote spawn batches (seconds).
pub(crate) const MOTE_SPAWN_INTERVAL: f32 = 0.15;
/// Number of motes per spawn batch.
pub(crate) const MOTE_COUNT_PER_SPAWN: usize = 2;

// ── Smoke poof (transformation/banishment effects) ──────────────────
/// How long each smoke poof lives (seconds).
pub(crate) const POOF_LIFETIME: f32 = 0.6;
/// Base size of smoke poof particles.
pub(crate) const POOF_SIZE: f32 = 8.0;
/// Spread speed of poof particles.
pub(crate) const POOF_SPREAD_SPEED: f32 = 40.0;
/// Rise speed of poof particles.
pub(crate) const POOF_RISE_SPEED: f32 = 20.0;
