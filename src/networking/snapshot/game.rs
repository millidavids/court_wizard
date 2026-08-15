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

/// Bit 0 of [`BeamSnapshot::flags`]: this beam was emitted by an Arcane Crystal
/// rather than cast by a wizard.
///
/// Crystal beams take the `spawn_beam_core` shape locally — core mesh only, no glow
/// cone and no origin flare. Without this bit the receiving peer could not tell the
/// two apart and attached the wizard visuals to both, which put a `FLARE_RADIUS`
/// (32-unit) opaque emissive sphere exactly on the crystal's own position. The
/// crystal's half-extents are around 8 × 17 × 8, so it vanished inside a glowing
/// ball — and because Disintegrate is the one infusion that keeps a beam alive for
/// the crystal's whole life, it stayed that way.
pub const BEAM_FLAG_FROM_CRYSTAL: u8 = 1 << 0;

/// Compact per-beam state (~29 bytes).
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
    /// Bit flags — see [`BEAM_FLAG_FROM_CRYSTAL`].
    pub flags: u8,
}
