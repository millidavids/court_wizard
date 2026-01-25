//! Fireball spell visual styles.
//!
//! Contains colors and visual parameters for fireball rendering.

use bevy::prelude::*;

/// Color of the fireball projectile (orange).
pub const FIREBALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.0);

/// Radius of the fireball projectile mesh.
pub const FIREBALL_RADIUS: f32 = 10.0;

/// Color of the explosion sphere (red-orange).
pub const EXPLOSION_COLOR: Color = Color::srgb(1.0, 0.3, 0.0);
