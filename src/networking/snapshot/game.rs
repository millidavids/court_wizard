//! Top-level game snapshot and projectile snapshot types.

use serde::{Deserialize, Serialize};

use super::unit::UnitSnapshot;

/// Unit state snapshot sent from host to guest each frame.
///
/// Contains only unit and unit-projectile data. Spell visuals are sent
/// separately via `SpellVisualSnapshot` (bidirectional).
#[derive(Serialize, Deserialize)]
pub struct GameSnapshot {
    /// Monotonically increasing tick counter for ordering.
    pub tick: u32,
    /// State of every tracked unit.
    pub units: Vec<UnitSnapshot>,
    /// State of every in-flight arrow projectile.
    pub arrows: Vec<ArrowSnapshot>,
    /// Host's authoritative match-elapsed seconds (`KillStats.elapsed_time`), so
    /// the guest's HUD clock mirrors the host exactly instead of free-running on
    /// its own local timer. Appended last (positional bincode; same-version MP).
    pub host_elapsed_secs: f32,
}

/// Compact per-arrow state (~12 bytes).
#[derive(Serialize, Deserialize)]
pub struct ArrowSnapshot {
    /// World position X.
    pub x: f32,
    /// World position Y (height).
    pub y: f32,
    /// World position Z.
    pub z: f32,
}

/// Compact per-magic-missile state (~12 bytes).
#[derive(Serialize, Deserialize)]
pub struct MagicMissileSnapshot {
    /// World position X.
    pub x: f32,
    /// World position Y (height).
    pub y: f32,
    /// World position Z.
    pub z: f32,
}

/// Compact per-beam state (~28 bytes).
///
/// Encodes origin, direction, and length for disintegrate and similar beams.
#[derive(Serialize, Deserialize)]
pub struct BeamSnapshot {
    /// Origin X.
    pub ox: f32,
    /// Origin Y.
    pub oy: f32,
    /// Origin Z.
    pub oz: f32,
    /// Direction X.
    pub dx: f32,
    /// Direction Y.
    pub dy: f32,
    /// Direction Z.
    pub dz: f32,
    /// Beam length.
    pub length: f32,
    /// Beam core width (`DisintegrateBeam::beam_width()`) so the ghost renders at
    /// the caster's real thickness instead of a hardcoded value.
    pub width: f32,
}
