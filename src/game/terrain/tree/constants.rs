use crate::game::constants::UNIT_SCALE;

/// Number of tree sprite variants in the sprite sheet.
pub(crate) const TREE_SPRITE_COUNT: usize = 5;

/// Visual width of a tree sprite in world units (64px * UNIT_SCALE * 2).
pub(super) const TREE_SPRITE_WIDTH: f32 = 128.0 * UNIT_SCALE;

/// Visual height of a tree sprite in world units (64px * UNIT_SCALE * 2).
pub(super) const TREE_SPRITE_HEIGHT: f32 = 128.0 * UNIT_SCALE;

/// Collision radius of a tree on the XZ plane (trunk is much narrower than canopy).
pub const TREE_RADIUS: f32 = TREE_SPRITE_WIDTH / 8.0;

/// Vertical extent for projectile collision checks.
pub const TREE_HEIGHT: f32 = TREE_SPRITE_HEIGHT;

/// How far the tree sinks into the ground (hides bottom edge).
pub(super) const TREE_GROUND_CLIP: f32 = 4.0 * UNIT_SCALE;

/// Shadow scale multiplier for trees.
pub(super) const TREE_SHADOW_SCALE: f32 = 5.0;

/// Returns the collision radius for a specific tree sprite variant.
/// Indices 2 and 3 (thick-trunk trees) get 1.5x the base radius.
pub const fn tree_radius_for_variant(sprite_index: u8) -> f32 {
    match sprite_index {
        2 | 3 => TREE_RADIUS * 1.5,
        _ => TREE_RADIUS,
    }
}

/// Multiplier on tree collision radius for the heat zone that applies FireDoT.
pub const BURNING_TREE_HEAT_RADIUS_MULTIPLIER: f32 = 2.0;

/// Effective spell damage used when stacking FireDoT from burning tree heat.
/// Each tick adds `this * FIRE_DOT_DAMAGE_RATIO` (0.1) = 1.5 DPS to the DoT.
pub const BURNING_TREE_HEAT_SPELL_DAMAGE: f32 = 15.0;

/// Tick interval for burning tree fire damage (seconds).
pub const BURNING_TREE_TICK_INTERVAL: f32 = 0.5;

/// Interval between fire smoke emissions from burning trees (seconds).
pub const BURNING_TREE_SMOKE_INTERVAL: f32 = 0.3;

/// Interval between spark emissions from burning trees (seconds).
pub const BURNING_TREE_SPARK_INTERVAL: f32 = 0.8;

/// Base number of trees spawned per level (before terrain density scaling).
pub const TREE_BASE_COUNT_MIN: u32 = 3;
pub const TREE_BASE_COUNT_MAX: u32 = 10;
