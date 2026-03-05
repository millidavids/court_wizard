use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// Entity Sizes
pub const UNIT_RADIUS: f32 = 8.0 * UNIT_SCALE; // Circle radius for units

// Sprite animation
pub const INFANTRY_SPRITE_WIDTH: f32 = 24.0 * UNIT_SCALE; // World-space quad width (~0.75 aspect ratio)
pub const INFANTRY_SPRITE_HEIGHT: f32 = 32.0 * UNIT_SCALE; // World-space quad height

// Sprite tint colors (applied to team-specific textures)
pub const DEFENDER_SPRITE_TINT: Color = Color::srgb(1.3, 1.3, 1.5);
pub const ATTACKER_SPRITE_TINT: Color = Color::srgb(0.55, 0.45, 0.45);
pub const KINGS_GUARD_SPRITE_TINT: Color = Color::srgb(1.2, 0.45, 0.35);
pub const UNDEAD_SPRITE_TINT: Color = Color::srgb(0.55, 0.35, 0.75);
