/// Minimum allocation percentage for any slider (prevents disabling stats completely)
pub(super) const MIN_ALLOCATION: f32 = 25.0;

/// Maximum allocation percentage for any slider (prevents over-specialization)
pub(super) const MAX_ALLOCATION: f32 = 200.0;

/// Default allocation percentage for each slider at start
pub(super) const DEFAULT_ALLOCATION: f32 = 100.0;

/// Total allocation pool shared across all sliders (4 sliders × 100%)
pub(super) const TOTAL_ALLOCATION_POOL: f32 = 400.0;
