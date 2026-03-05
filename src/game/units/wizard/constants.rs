//! Wizard unit constants.
//!
//! Contains all hardcoded values for wizard behavior.

/// Wizard hitbox radius (half sprite width).
pub const HITBOX_RADIUS: f32 = WIZARD_SPRITE_WIDTH / 2.0;

/// Wizard hitbox height (matches sprite height).
pub const HITBOX_HEIGHT: f32 = WIZARD_SPRITE_HEIGHT;

/// Wizard starting health.
pub const HEALTH: f32 = 200.0;

/// Wizard starting mana.
pub const MANA: f32 = 100.0;

/// Wizard mana regeneration per second.
pub const MANA_REGEN: f32 = 5.0;

/// Wizard default spell range (units from wizard).
pub const DEFAULT_SPELL_RANGE: f32 = 3000.0;

// Sprite sheet animation parameters
/// Total animation frames in the idle sprite sheet.
pub const WIZARD_SPRITE_FRAMES: usize = 9;
/// Grid dimension (3x3 grid).
pub const WIZARD_SPRITE_GRID_SIZE: usize = 3;
/// Width of each frame in pixels.
pub const WIZARD_FRAME_WIDTH: f32 = 126.0;
/// Height of each frame in pixels.
pub const WIZARD_FRAME_HEIGHT: f32 = 132.0;
/// Full idle animation loop duration in seconds.
pub const WIZARD_ANIMATION_LOOP_DURATION: f32 = 2.0;
/// Duration of each animation frame.
pub const WIZARD_FRAME_DURATION: f32 = WIZARD_ANIMATION_LOOP_DURATION / WIZARD_SPRITE_FRAMES as f32;
/// World-space sprite size (2x pixel dimensions for visibility at camera distance).
pub const WIZARD_SPRITE_WIDTH: f32 = WIZARD_FRAME_WIDTH * 2.0;
pub const WIZARD_SPRITE_HEIGHT: f32 = WIZARD_FRAME_HEIGHT * 2.0;
