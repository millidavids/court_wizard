//! Teleport VFX tuneable constants.

// ── Spatial distortion (screen-space ripple) ─────────────────────────
/// Duration of the one-shot ripple effect (seconds).
pub(crate) const RIPPLE_DURATION: f32 = 0.4;
/// Global distortion strength for one-shot teleport ripples.
pub(crate) const RIPPLE_STRENGTH: f32 = 0.0015;
/// Wave frequency (number of concentric rings within the influence radius).
pub(crate) const RIPPLE_FREQUENCY: f32 = 10.0;
/// Wave propagation speed.
pub(crate) const RIPPLE_SPEED: f32 = 6.0;
/// Starting intensity of one-shot ripples (decays to 0 over duration).
pub(crate) const RIPPLE_INTENSITY: f32 = 0.8;

/// Global distortion strength for persistent Dimensional Rift ripples.
pub(crate) const RIFT_RIPPLE_STRENGTH: f32 = 0.001;
/// Intensity of persistent Dimensional Rift ripples (constant, not decaying).
pub(crate) const RIFT_RIPPLE_INTENSITY: f32 = 0.4;

/// Multiplier from world-space radius to screen-space influence radius.
pub(crate) const RIPPLE_INFLUENCE_MULT: f32 = 2.0;

// ── Dimensional Rift lensing ──────────────────────────────────────────
/// World-space radius for rift lensing influence (small, like a mini black hole).
pub(crate) const RIFT_LENSING_RADIUS: f32 = 25.0;
