use bevy::prelude::*;

// Arrow
pub const ARROW_COLOR: Color = Color::srgb(0.45, 0.3, 0.15); // Brown
pub const ARCHER_RADIUS: f32 = 8.0; // Same as infantry

// Sprite animation
pub const ARCHER_SPRITE_WIDTH: f32 = 24.0; // World-space quad width (~0.75 aspect ratio)
pub const ARCHER_SPRITE_HEIGHT: f32 = 32.0; // World-space quad height

// Sprite tint colors
pub use crate::game::units::infantry::styles::DEFENDER_SPRITE_TINT;
/// Lighter attacker tint for archers (infantry uses darker 0.55/0.45/0.45).
pub const ATTACKER_SPRITE_TINT: Color = Color::srgb(0.75, 0.65, 0.65);
