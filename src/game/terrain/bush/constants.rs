use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// Sprite sheet (bush_tiles.png: 10 sprites in a row, each 32x32 pixels)
/// Number of bush sprite variants in the sprite sheet.
pub(crate) const BUSH_SPRITE_COUNT: usize = 10;
/// Width of a single bush sprite in world units.
pub(super) const BUSH_SPRITE_WIDTH: f32 = 32.0 * UNIT_SCALE;
/// Height of a single bush sprite in world units.
pub(super) const BUSH_SPRITE_HEIGHT: f32 = 32.0 * UNIT_SCALE;

/// Collision radius of a bush on the XZ plane (matches sprite half-width).
pub const BUSH_RADIUS: f32 = BUSH_SPRITE_WIDTH / 2.0;

/// How far the sprite sinks into the ground (a couple of pixels).
pub(super) const BUSH_GROUND_CLIP: f32 = 2.0 * UNIT_SCALE;

/// Shadow scale relative to bush sprite.
pub(super) const BUSH_SHADOW_SCALE: f32 = 1.6;

/// Fire damage per second dealt by a burning bush to units inside.
pub const BURNING_BUSH_DPS: f32 = 15.0;

/// Tick interval for burning bush fire damage (seconds).
pub const BURNING_BUSH_TICK_INTERVAL: f32 = 0.5;

/// Orange tint overlay applied to burning bushes (keeps sprite visible underneath).
pub const BURNING_BUSH_TINT: Color = Color::srgba(1.0, 0.55, 0.25, 1.0);

/// Flow field cost for burning bush cells (hazardous — strongly avoided).
pub const BURNING_BUSH_FLOW_COST: f32 = 15.0;

/// Interval between fire smoke emissions from burning bushes (seconds).
pub const BURNING_BUSH_SMOKE_INTERVAL: f32 = 0.4;

/// Interval between spark emissions from burning bushes (seconds).
pub const BURNING_BUSH_SPARK_INTERVAL: f32 = 1.2;

/// Flow field cost for bush cells (units path around but can push through).
pub const BUSH_FLOW_COST: f32 = 3.0;

/// Speed modifier applied to units inside a bush (-25% speed).
pub const BUSH_SPEED_MODIFIER: f32 = -0.25;

/// Base number of bushes spawned per level (before terrain density scaling).
pub const BUSH_BASE_COUNT_MIN: u32 = 4;
pub const BUSH_BASE_COUNT_MAX: u32 = 12;
