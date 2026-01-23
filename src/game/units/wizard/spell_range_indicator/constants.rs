//! Spell range indicator visual constants.

use bevy::prelude::*;

/// Color of the spell range dots (light blue).
pub const RANGE_DOT_COLOR: Color = Color::srgb(0.5, 0.8, 1.0);

/// Size of each dot (width and height in units).
pub const DOT_SIZE: f32 = 8.0;

/// Number of dots in the dotted circle.
pub const NUM_DOTS: usize = 256;

/// Rotation speed of the circle (radians per second).
pub const ROTATION_SPEED: f32 = 0.0625;
