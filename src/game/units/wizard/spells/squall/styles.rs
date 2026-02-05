//! Squall spell visual styles.
//!
//! Contains colors and visual parameters for squall ice effects.

use bevy::prelude::*;

/// Color of the storm circle indicator (cyan/blue).
pub const CIRCLE_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.4);

/// Color of the ice projectile (light blue/white).
pub const ICE_PROJECTILE_COLOR: Color = Color::srgb(0.7, 0.9, 1.0);

/// Color of the ice explosion sphere (bright cyan/blue).
pub const EXPLOSION_COLOR: Color = Color::srgb(0.3, 0.8, 1.0);
