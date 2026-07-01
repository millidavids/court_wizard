/// Allocation units a slider moves per key press / D-pad press. Shared by the
/// keyboard handler and the controller path so the two can't drift (the range is
/// [MIN_ALLOCATION, MAX_ALLOCATION], i.e. 25–200).
pub(crate) const SLIDER_KEY_STEP: f32 = 10.0;

/// Minimum allocation percentage for any slider (prevents disabling stats completely)
pub(super) const MIN_ALLOCATION: f32 = 25.0;

/// Maximum allocation percentage for any slider (prevents over-specialization)
pub(super) const MAX_ALLOCATION: f32 = 200.0;

/// Default allocation percentage for each slider at start
pub(super) const DEFAULT_ALLOCATION: f32 = 100.0;

/// Total allocation pool shared across all sliders (4 sliders × 100%)
pub(super) const TOTAL_ALLOCATION_POOL: f32 = 400.0;
