use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// Arrow
pub const ARROW_COLOR: Color = Color::srgb(0.3, 0.2, 0.1); // Dark brown
pub const ARCHER_RADIUS: f32 = 8.0 * UNIT_SCALE; // Same as infantry

// Sprite animation (re-export shared defaults for backward compatibility)
pub use crate::game::units::constants::DEFAULT_SPRITE_WIDTH as ARCHER_SPRITE_WIDTH;
pub use crate::game::units::constants::DEFAULT_SPRITE_HEIGHT as ARCHER_SPRITE_HEIGHT;

// Sprite tint colors
pub use crate::game::units::infantry::styles::DEFENDER_SPRITE_TINT;
/// Lighter attacker tint for archers (infantry uses darker 0.55/0.45/0.45).
pub const ATTACKER_SPRITE_TINT: Color = Color::srgb(0.75, 0.65, 0.65);
