/// Horizontal sway amplitude in UV space (how far the top of a tree sprite shifts).
pub(crate) const WIND_SWAY_AMPLITUDE: f32 = 0.01;

/// Sway oscillation speed in radians per second.
pub(crate) const WIND_SWAY_SPEED: f32 = 0.5;

/// Bush sway amplitude (lighter foliage, less sway than trees).
pub(crate) const BUSH_SWAY_AMPLITUDE: f32 = WIND_SWAY_AMPLITUDE * 0.7;

/// Bush sway speed (lighter foliage oscillates faster).
pub(crate) const BUSH_SWAY_SPEED: f32 = WIND_SWAY_SPEED * 1.2;

/// Flora sway amplitude (tiny sprites, very subtle).
pub(crate) const FLORA_SWAY_AMPLITUDE: f32 = WIND_SWAY_AMPLITUDE * 0.5;

/// Flora sway speed (grassy, quick oscillation).
pub(crate) const FLORA_SWAY_SPEED: f32 = WIND_SWAY_SPEED * 1.5;
