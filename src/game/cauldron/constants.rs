use bevy::prelude::*;

use crate::game::constants::{WIZARD_2_POSITION, WIZARD_POSITION};

/// Offset from wizard position to place the cauldron beside the wizard on the castle wall.
const CAULDRON_OFFSET: Vec3 = Vec3::new(60.0, -64.0, 90.0);

/// Cauldron position in 3D space (on castle platform, next to wizard).
pub const CAULDRON_POSITION: Vec3 = Vec3::new(
    WIZARD_POSITION.x + CAULDRON_OFFSET.x,
    WIZARD_POSITION.y + CAULDRON_OFFSET.y,
    WIZARD_POSITION.z + CAULDRON_OFFSET.z,
);

/// Cauldron position for the multiplayer GUEST, beside `WIZARD_2_POSITION`.
/// Mirrored across the origin (x/z negated) the same way `SPELL_2_OFFSET` mirrors
/// `SPELL_OFFSET`, so the guest's cauldron sits at their own castle.
pub const CAULDRON_2_POSITION: Vec3 = Vec3::new(
    WIZARD_2_POSITION.x - CAULDRON_OFFSET.x,
    WIZARD_2_POSITION.y + CAULDRON_OFFSET.y,
    WIZARD_2_POSITION.z - CAULDRON_OFFSET.z,
);

/// Shared cauldron position for **co-op** — right next to the MAIN (host) wizard,
/// on its wall ledge (the same spot as the host's single-player cauldron). Used by
/// BOTH the co-op host (SP loading path) and the co-op guest so neither peer sees
/// it move; the guest stands further along the wall to its right.
pub const CAULDRON_COOP_POSITION: Vec3 = Vec3::new(
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

/// Sprite sheet animation parameters
pub const CAULDRON_SPRITE_FRAMES: usize = 9;
pub const CAULDRON_SPRITE_GRID_SIZE: usize = 3; // 3x3 grid
pub const CAULDRON_ANIMATION_LOOP_DURATION: f32 = 2.0; // 2 second loop
pub const CAULDRON_FRAME_DURATION: f32 =
    CAULDRON_ANIMATION_LOOP_DURATION / CAULDRON_SPRITE_FRAMES as f32;

/// Size of the cauldron billboard (pixels in world space)
pub const CAULDRON_SPRITE_SIZE: f32 = 96.0;

/// Brewing visual effect parameters
pub const BREWING_PULSE_SCALE_MIN: f32 = 0.95; // Shrink to 95% of normal size
pub const BREWING_PULSE_SCALE_MAX: f32 = 1.05; // Grow to 105% of normal size
pub const BREWING_PULSE_DURATION: f32 = 1.0; // 1 second per pulse cycle
pub const BREWING_COLOR_ALPHA_MIN: f32 = 0.3; // Min color opacity
pub const BREWING_COLOR_ALPHA_MAX: f32 = 0.9; // Max color opacity

// Pre-calculated ranges for brewing effects
pub const BREWING_PULSE_SCALE_RANGE: f32 = BREWING_PULSE_SCALE_MAX - BREWING_PULSE_SCALE_MIN;
pub const BREWING_COLOR_ALPHA_RANGE: f32 = BREWING_COLOR_ALPHA_MAX - BREWING_COLOR_ALPHA_MIN;
