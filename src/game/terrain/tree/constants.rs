use bevy::prelude::*;

/// Visual width of the tree rectangle (placeholder brown rectangle).
pub const TREE_VISUAL_WIDTH: f32 = 40.0;

/// Visual height of the tree rectangle (placeholder brown rectangle).
pub const TREE_VISUAL_HEIGHT: f32 = 60.0;

/// Collision radius of a tree on the XZ plane (matches visual half-width).
pub const TREE_RADIUS: f32 = TREE_VISUAL_WIDTH / 2.0;

/// Vertical extent for projectile collision checks.
pub const TREE_HEIGHT: f32 = TREE_VISUAL_HEIGHT;

/// Tree color (brown, placeholder until sprites are drawn).
pub const TREE_COLOR: Color = Color::srgb(0.45, 0.30, 0.15);

/// Hit points for a tree. Destroyed only by fire damage.
pub const TREE_HEALTH: f32 = 150.0;

/// Base number of trees spawned per level (before terrain density scaling).
pub const TREE_BASE_COUNT_MIN: u32 = 3;
pub const TREE_BASE_COUNT_MAX: u32 = 10;
