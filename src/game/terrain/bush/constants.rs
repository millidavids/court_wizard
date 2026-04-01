use bevy::prelude::*;

/// Collision radius of a bush on the XZ plane (world units).
pub const BUSH_RADIUS: f32 = 28.0;

/// Visual radius of the bush circle mesh.
pub const BUSH_VISUAL_RADIUS: f32 = 28.0;

/// Bush color (green).
pub const BUSH_COLOR: Color = Color::srgb(0.20, 0.50, 0.20);

/// Fire damage per second dealt by a burning bush to units inside.
pub const BURNING_BUSH_DPS: f32 = 15.0;

/// Tick interval for burning bush fire damage (seconds).
pub const BURNING_BUSH_TICK_INTERVAL: f32 = 0.5;

/// Color of a burning bush (orange-red).
pub const BURNING_BUSH_COLOR: Color = Color::srgb(0.85, 0.35, 0.10);

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

/// Height of the bush (for visual purposes; bushes don't block projectiles).
pub const BUSH_HEIGHT: f32 = 40.0;

