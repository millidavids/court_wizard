use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// Entity Sizes
pub const UNIT_RADIUS: f32 = 8.0 * UNIT_SCALE; // Circle radius for units

// Sprite animation (re-export shared defaults for backward compatibility)
pub use crate::game::units::constants::DEFAULT_SPRITE_HEIGHT as INFANTRY_SPRITE_HEIGHT;
pub use crate::game::units::constants::DEFAULT_SPRITE_WIDTH as INFANTRY_SPRITE_WIDTH;

// Sprite tint colors (applied to team-specific textures)
pub const DEFENDER_SPRITE_TINT: Color = Color::srgb(1.3, 1.3, 1.5);
pub const ATTACKER_SPRITE_TINT: Color = Color::srgb(0.55, 0.45, 0.45);
pub const KINGS_GUARD_SPRITE_TINT: Color = Color::srgb(1.2, 0.45, 0.35);
pub const UNDEAD_SPRITE_TINT: Color = Color::srgb(0.7, 0.55, 0.75);
