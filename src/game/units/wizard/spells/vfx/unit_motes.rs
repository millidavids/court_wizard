//! Shared cadence and scatter math for per-unit mote emitters.
//!
//! Used by buff visuals that trickle motes off individual units (Battle Hymn
//! song motes, Healing Plume regen motes). The cadence gate is phase-offset
//! per entity so a whole buffed army doesn't pulse in unison, and the scatter
//! helper turns the resulting seed into a horizontal offset around the unit.

use bevy::prelude::*;

/// Golden-ratio fraction used to derive a stable per-entity phase offset.
const PHASE_GOLDEN_FRACTION: f32 = 0.618_034;

/// Per-entity phase-offset cadence gate.
///
/// Returns `Some(seed)` on the frame this entity's offset timer crosses an
/// `interval` boundary (i.e. "emit now"), `None` otherwise. The seed is
/// stable-per-entity but time-varying, suitable for feeding
/// [`mote_scatter_offset`] and other pseudo-random particle parameters.
///
/// Prefer this over a system-wide `Local<f32>` accumulator for per-unit
/// emitters: it staggers spawns across frames instead of bursting the whole
/// army on one frame, and being stateless it cannot drift or run away if the
/// frame rate collapses below the emit interval.
pub(crate) fn unit_effect_tick(
    entity: Entity,
    elapsed: f32,
    dt: f32,
    interval: f32,
) -> Option<f32> {
    let entity_seed = entity.index().index() as f32;
    let phase = (entity_seed * PHASE_GOLDEN_FRACTION).fract() * interval;
    let tp = elapsed + phase;
    if (tp / interval).floor() == ((tp - dt) / interval).floor() {
        return None;
    }
    Some(entity_seed * 1.618_034 + elapsed * 9.3)
}

/// Horizontal scatter offset around a unit derived from a mote seed.
///
/// `min_frac` is the minimum fraction of `radius` used, so motes never spawn
/// dead-center on the sprite.
pub(crate) fn mote_scatter_offset(seed: f32, radius: f32, min_frac: f32) -> Vec2 {
    let angle = seed * 2.39 + (seed * 11.7).sin() * 1.3;
    let r = radius * (min_frac + (1.0 - min_frac) * ((seed * 19.1).sin() * 0.5 + 0.5));
    Vec2::new(angle.cos() * r, angle.sin() * r)
}
