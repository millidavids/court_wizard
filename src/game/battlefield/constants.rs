use bevy::prelude::*;

use crate::game::constants::*;

// ===== Battlefield =====

/// Battlefield ground plane color (#346D54).
pub(crate) const BATTLEFIELD_COLOR: Color = Color::srgb(0.204, 0.427, 0.329);

// ===== Battlefield Tile Constants =====

/// World size of each ground tile (square).
pub(super) const TILE_WORLD_SIZE: f32 = 50.0;

/// Number of tile variants in the sprite sheet.
pub(super) const TILE_COUNT: usize = 10;

/// Number of base (grass) tiles at the start of the sprite sheet.
pub(super) const TILE_BASE_COUNT: usize = 5;

/// Selection weight for base tiles (indices 0-4).
pub(super) const TILE_BASE_WEIGHT: u32 = 40;

/// Selection weight for flavor tiles (indices 5-9).
pub(super) const TILE_FLAVOR_WEIGHT: u32 = 1;

/// Total weight for weighted random tile selection.
pub(super) const TILE_TOTAL_WEIGHT: u32 = TILE_BASE_COUNT as u32 * TILE_BASE_WEIGHT
    + (TILE_COUNT - TILE_BASE_COUNT) as u32 * TILE_FLAVOR_WEIGHT;

// ===== Right Wall Constants =====

/// The right wall image is 320x180 (16:9 aspect ratio).
const RIGHT_WALL_IMAGE_WIDTH: f32 = 320.0;
const RIGHT_WALL_IMAGE_HEIGHT: f32 = 180.0;

/// Width of the right wall backdrop in world units.
pub(super) const RIGHT_WALL_WIDTH: f32 = 4000.0;

/// Height derived from aspect ratio.
pub(super) const RIGHT_WALL_HEIGHT: f32 =
    RIGHT_WALL_WIDTH * (RIGHT_WALL_IMAGE_HEIGHT / RIGHT_WALL_IMAGE_WIDTH);

/// Position of the right wall (along the +X edge of the battlefield, facing inward).
/// Centered along the Z axis, raised so it fills the background.
/// Positioned so the bottom-left corner of the image meets the top corner of the battlefield.
/// Bottom edge at Y=0 (center Y = half height), far end at Z = -BATTLEFIELD_HALF
/// (center Z = -BATTLEFIELD_HALF + half width).
const BATTLEFIELD_HALF: f32 = BATTLEFIELD_SIZE / 2.0;
pub(super) const RIGHT_WALL_POSITION: Vec3 = Vec3::new(
    BATTLEFIELD_HALF,
    RIGHT_WALL_HEIGHT / 2.0,
    -BATTLEFIELD_HALF + RIGHT_WALL_WIDTH / 2.0,
);

/// Rotation so the wall faces inward (toward -X).
/// A Rectangle mesh faces +Z by default, so rotate 90° to face -X.
pub(super) const RIGHT_WALL_ROTATION_DEGREES: f32 = -90.0;

// ===== Left (Back) Wall Constants =====

/// The left wall image is 640x180 (twice as wide as the right wall).
const LEFT_WALL_IMAGE_WIDTH: f32 = 640.0;
const LEFT_WALL_IMAGE_HEIGHT: f32 = 180.0;

/// Height matches the right wall.
pub(super) const LEFT_WALL_HEIGHT: f32 = RIGHT_WALL_HEIGHT;

/// Width derived from aspect ratio to match the right wall height.
pub(super) const LEFT_WALL_WIDTH: f32 =
    LEFT_WALL_HEIGHT * (LEFT_WALL_IMAGE_WIDTH / LEFT_WALL_IMAGE_HEIGHT);

/// Position of the left wall (along the -Z edge of the battlefield, facing +Z toward camera).
/// Shares the corner with the right wall at (BATTLEFIELD_HALF, 0, -BATTLEFIELD_HALF).
/// Extends from that corner to the left along the X axis.
pub(super) const LEFT_WALL_POSITION: Vec3 = Vec3::new(
    BATTLEFIELD_HALF - LEFT_WALL_WIDTH / 2.0,
    LEFT_WALL_HEIGHT / 2.0,
    -BATTLEFIELD_HALF,
);

// ===== Wall Floor Constants =====

/// The wall floor image is 180x320 (right wall dimensions transposed).
const WALL_FLOOR_IMAGE_WIDTH: f32 = 180.0;
const WALL_FLOOR_IMAGE_HEIGHT: f32 = 320.0;

/// The long dimension matches the right wall width (along Z).
pub(crate) const WALL_FLOOR_LENGTH: f32 = RIGHT_WALL_WIDTH; // 4000
/// The short dimension matches the right wall height (extends inward along X).
pub(crate) const WALL_FLOOR_DEPTH: f32 =
    WALL_FLOOR_LENGTH * (WALL_FLOOR_IMAGE_WIDTH / WALL_FLOOR_IMAGE_HEIGHT);

/// Position: lies flat in the corner where right wall and left wall meet.
/// One long edge at X=BATTLEFIELD_HALF (right wall base), one short edge at Z=-BATTLEFIELD_HALF (left wall base).
pub(crate) const WALL_FLOOR_POSITION: Vec3 = Vec3::new(
    BATTLEFIELD_HALF - WALL_FLOOR_DEPTH / 2.0,
    1.0,
    -BATTLEFIELD_HALF + WALL_FLOOR_LENGTH / 2.0,
);

// ===== Environmental Feature Positions =====
// Derived from pixel positions on wall_floor.png (180x320).
// Pixel (x, y) → World X = 750 + (x/180) * 2250, Z = -3000 + (y/320) * 4000

/// World position of the lava pool (approx pixel 120, 210).
pub(crate) const LAVA_POOL_POSITION: Vec3 = Vec3::new(2500.0, 2.0, -750.0);
/// Radius of the lava pool for particle spread, visual effects, and damage.
pub(crate) const LAVA_POOL_RADIUS: f32 = 300.0;
/// Radius within which units actually take lava damage (matches visual).
pub(crate) const LAVA_DAMAGE_RADIUS: f32 = LAVA_POOL_RADIUS - 40.0;
/// Radius used for flow field avoidance (20 units past the damage edge).
pub(crate) const LAVA_AVOIDANCE_RADIUS: f32 = LAVA_POOL_RADIUS + 40.0;

/// World position of the water pool (approx pixel 50, 55).
pub(crate) const WATER_POOL_POSITION: Vec3 = Vec3::new(2075.0, 3.0, -2510.0);
/// Approximate radius of the water pool for ripple effect.
pub(crate) const WATER_POOL_RADIUS: f32 = 700.0;

/// Interval between lava fire smoke spawns (seconds).
pub(super) const LAVA_SMOKE_INTERVAL: f32 = 0.3;
/// Interval between lava spark bursts (seconds).
pub(super) const LAVA_SPARK_INTERVAL: f32 = 1.5;

/// Interval between water ripple spawns (seconds).
pub(super) const WATER_RIPPLE_INTERVAL: f32 = 1.2;
/// Maximum scale a ripple grows to (world units).
pub(super) const WATER_RIPPLE_MAX_SCALE: f32 = 200.0;
/// How long each ripple lives (seconds).
pub(super) const WATER_RIPPLE_LIFETIME: f32 = 4.0;
/// Peak alpha for ripples.
pub(super) const WATER_RIPPLE_ALPHA: f32 = 0.35;

// ===== Terrain Hazard Flow Field Costs =====

/// Flow field cost for lava pool cells. Very high to strongly discourage pathing.
pub(crate) const LAVA_HAZARD_FLOW_COST: f32 = 50.0;
/// Flow field cost for water pool cells. Moderate to mildly discourage pathing.
pub(crate) const WATER_SLOW_FLOW_COST: f32 = 3.0;

/// Lava damage per second to units inside the pool.
pub(crate) const LAVA_DAMAGE_PER_SECOND: f32 = 100.0;
/// Water speed reduction applied as a percentage modifier (e.g., -0.5 = 50% slower).
pub(crate) const WATER_SPEED_MODIFIER: f32 = -0.5;

// ===== Ground Noise Constants =====

/// Base noise intensity for the open battlefield.
pub(super) const GROUND_NOISE_BASE: f32 = 0.6;
/// Noise intensity under forest tree concentration (more earthy).
pub(super) const GROUND_NOISE_FOREST: f32 = 2.0;
/// Z threshold below which full forest noise intensity applies.
pub(super) const GROUND_NOISE_FOREST_Z: f32 = -600.0;
/// Z range over which noise blends between forest and base intensity.
/// The blend edge is further randomized by noise in the shader.
pub(super) const GROUND_NOISE_BLEND_RANGE: f32 = 1200.0;

// ===== Ambient Mote Constants =====

/// Interval between ambient mote spawn batches (seconds).
pub(super) const AMBIENT_MOTE_INTERVAL: f32 = 0.5;
/// Number of motes to spawn per batch.
pub(super) const AMBIENT_MOTE_COUNT: usize = 3;
/// Minimum X for ambient mote spawn region.
pub(super) const AMBIENT_MOTE_MIN_X: f32 = -2200.0;
/// Maximum X for ambient mote spawn region.
pub(super) const AMBIENT_MOTE_MAX_X: f32 = 2000.0;
/// Minimum Z for ambient mote spawn region.
pub(super) const AMBIENT_MOTE_MIN_Z: f32 = -2200.0;
/// Maximum Z for ambient mote spawn region.
pub(super) const AMBIENT_MOTE_MAX_Z: f32 = 2200.0;
