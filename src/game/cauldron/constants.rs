use bevy::prelude::*;

use crate::game::constants::WIZARD_POSITION;

/// Cauldron color (charcoal) - DEPRECATED: Now using sprite sheet.
#[allow(dead_code)]
pub const CAULDRON_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

/// Visual radius of the cauldron circle - DEPRECATED: Now using sprite sheet.
#[allow(dead_code)]
pub const CAULDRON_RADIUS: f32 = 20.0;

/// Offset from wizard position to place the cauldron beside the wizard on the castle wall.
const CAULDRON_OFFSET: Vec3 = Vec3::new(-60.0, 0.0, -60.0);

/// Cauldron position in 3D space (on castle platform, next to wizard).
pub const CAULDRON_POSITION: Vec3 = Vec3::new(
    WIZARD_POSITION.x + CAULDRON_OFFSET.x,
    WIZARD_POSITION.y + CAULDRON_OFFSET.y,
    WIZARD_POSITION.z + CAULDRON_OFFSET.z,
);

/// Total duration of the brew bubble effect (seconds).
pub const BREW_BUBBLE_DURATION: f32 = 1.0;

/// Expansion speed of the bubble (units per second).
pub const BREW_BUBBLE_EXPAND_SPEED: f32 = 3000.0;

/// Starting alpha (translucency) of the bubble.
pub const BREW_BUBBLE_INITIAL_ALPHA: f32 = 0.2;

/// Y position of the bubble center.
pub const BREW_BUBBLE_HEIGHT: f32 = CAULDRON_POSITION.y;

/// Sprite sheet animation parameters
pub const CAULDRON_SPRITE_FRAMES: usize = 9;
pub const CAULDRON_SPRITE_GRID_SIZE: usize = 3; // 3x3 grid
pub const CAULDRON_ANIMATION_LOOP_DURATION: f32 = 2.0; // 2 second loop
pub const CAULDRON_FRAME_DURATION: f32 =
    CAULDRON_ANIMATION_LOOP_DURATION / CAULDRON_SPRITE_FRAMES as f32;

/// Size of the cauldron billboard (pixels in world space)
pub const CAULDRON_SPRITE_SIZE: f32 = 64.0;
