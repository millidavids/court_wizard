//! Shared visual effect constants.

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
pub(crate) const EXPLOSION_SMOKE_COUNT: usize = 6;
/// Lifetime of explosion smoke wisps (seconds).
pub(crate) const EXPLOSION_SMOKE_LIFETIME: f32 = 1.2;
/// Size of explosion smoke triangles.
pub(crate) const EXPLOSION_SMOKE_SIZE: f32 = 6.0;
/// Rise speed of explosion smoke.
pub(crate) const EXPLOSION_SMOKE_RISE_SPEED: f32 = 80.0;
/// Lateral spread of explosion smoke.
pub(crate) const EXPLOSION_SMOKE_SPREAD: f32 = 60.0;

// ── Impact sparks ────────────────────────────────────────────────────
/// Number of sparks spawned per explosion.
pub(crate) const SPARK_COUNT: usize = 10;
/// How long each spark lives (seconds).
pub(crate) const SPARK_LIFETIME: f32 = 0.4;
/// Base size of each spark triangle.
pub(crate) const SPARK_SIZE: f32 = 2.5;
/// Speed of sparks flying outward from impact.
pub(crate) const SPARK_SPEED: f32 = 250.0;

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
