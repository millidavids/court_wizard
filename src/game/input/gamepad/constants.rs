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

/// Seconds after a device-source event before switching the active input device.
/// Small hysteresis prevents flicker when the user bumps a stick or key accidentally.
pub(super) const DEVICE_SWITCH_STICK_MAGNITUDE: f32 = 0.25;
