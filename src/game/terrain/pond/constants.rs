use bevy::prelude::*;

/// Pond color (deep blue, sampled from wall_floor.png water pool center).
pub const POND_COLOR: Color = Color::srgba(0.13, 0.35, 0.65, 0.7);

/// Minimum radius for a pond (world units).
pub const POND_RADIUS_MIN: f32 = 80.0;

/// Maximum radius for a pond (world units).
pub const POND_RADIUS_MAX: f32 = 120.0;

/// Flow field cost for pond cells.
pub const POND_FLOW_COST: f32 = 2.5;

/// Electric damage multiplier for wet units (50% more electric arc damage).
pub const WET_ELECTRIC_DAMAGE_MULTIPLIER: f32 = 1.5;

/// Base number of ponds per level (before terrain density scaling).
pub const POND_BASE_COUNT_MIN: u32 = 1;
pub const POND_BASE_COUNT_MAX: u32 = 4;

/// Y position for pond surface (slightly above ground).
pub const POND_SURFACE_Y: f32 = 2.0;

/// Interval between ripple spawns per pond (seconds).
pub const POND_RIPPLE_INTERVAL: f32 = 1.5;

/// How long each pond ripple lives (seconds).
pub const POND_RIPPLE_LIFETIME: f32 = 3.0;

/// Peak alpha for pond ripples.
pub const POND_RIPPLE_ALPHA: f32 = 0.35;
