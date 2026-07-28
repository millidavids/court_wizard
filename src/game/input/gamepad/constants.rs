//! Gamepad input tuning constants.

/// Default stick deadzone (magnitude below which stick input is ignored).
pub(super) const DEFAULT_DEADZONE: f32 = 0.15;

/// Default cursor sensitivity in logical pixels per second at full stick deflection (X axis).
pub(crate) const DEFAULT_SENSITIVITY_X: f32 = 1200.0;

/// Default cursor sensitivity in logical pixels per second at full stick deflection (Y axis).
pub(crate) const DEFAULT_SENSITIVITY_Y: f32 = 1200.0;

/// Default response-curve exponent. 1.0 = linear, higher = more ease-out at low deflections.
pub(super) const DEFAULT_RESPONSE_CURVE: f32 = 2.2;

/// Trigger threshold (0..=1) for treating a trigger as a digital "press".
pub(super) const TRIGGER_THRESHOLD: f32 = 0.5;

/// Number of radial action bar slots.
pub(crate) const RADIAL_SLOT_COUNT: u8 = 5;

/// Angular width of each radial slot wedge (360° / slot count = 72°).
pub(crate) const RADIAL_WEDGE_DEGREES: f32 = 360.0 / RADIAL_SLOT_COUNT as f32;

/// Stick deflection away from its tracked resting baseline (summed over all
/// four axes, squared-magnitude compare) past which a stick shows device-claim
/// intent. Deflection is measured relative to the baseline — never as absolute
/// position — so a worn stick resting off-center (drift) can't claim focus.
/// Enforced for BOTH producers (gilrs and Steam Input) via
/// [`super::arbiter::StickIntentTracker`].
pub(super) const DEVICE_SWITCH_STICK_MAGNITUDE: f32 = 0.25;

/// Absolute per-axis deflection that always counts as stick intent regardless
/// of the baseline: no stick drifts to near-full throw, and a deliberate push
/// slow enough for the baseline to chase (smooth assistive controllers) still
/// reaches this.
pub(super) const STICK_ABSOLUTE_INTENT: f32 = 0.85;

/// Seconds without any mouse/keyboard event before *stick* intent may claim
/// input focus. Mouse use always has event-free frames between movements; this
/// grace window stops a noisy idle pad from stealing focus during them (the
/// cause of UI flashing when a drifting pad sat plugged in during mouse play).
/// Button edges bypass this window — a deliberate press must always register.
pub(super) const DEVICE_SWITCH_GRACE_SECS: f64 = 0.35;

/// Time constant (seconds) for the exponential moving average tracking each
/// stick's resting position. Slow enough that deliberate stick motion reads as
/// deflection from baseline before the baseline catches up, fast enough to
/// absorb gradual drift as a pad warms up mid-session.
pub(super) const STICK_BASELINE_TAU_SECS: f32 = 2.0;

/// Per-frame cap on the baseline EMA time step. Without it a long frame hitch
/// (alt-tab, loading) drives the EMA alpha to 1.0 and snaps the baseline onto
/// whatever position the stick happened to hold through the hitch.
pub(super) const BASELINE_MAX_STEP_SECS: f32 = 0.1;
