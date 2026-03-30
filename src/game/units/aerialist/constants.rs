use crate::game::constants::UNIT_SCALE;

// Movement
pub const AERIALIST_MOVEMENT_SPEED: f32 = 200.0;

// Momentum — aerialists never stop and turn in wide arcs
/// Minimum speed an aerialist will ever fly at (even with no target).
pub const AERIALIST_MIN_SPEED: f32 = 200.0;
/// How quickly the aerialist can change direction (radians per second).
/// Lower = wider sweeping arcs. Infantry-style steering is ~instant.
pub const AERIALIST_TURN_RATE: f32 = 1.5;

// Combat (aerialists fire archer arrows, so damage comes from ARCHER_ATTACK_DAMAGE)
#[allow(dead_code)]
pub const AERIALIST_ATTACK_DAMAGE: f32 = 15.0;
// 3D distance range — accounts for Y=144 fly height to ground (~124 Y gap).
// At ground level this gives ~sqrt(150²-124²) ≈ 84 XZ units of reach.
pub const AERIALIST_ATTACK_RANGE: f32 = 150.0;
// Flying height — bottom of sprite must clear Wall of Stone (height 80).
// Sprite half-height is 64, so center = 80 + 64 = 144.
pub const AERIALIST_FLY_HEIGHT: f32 = 144.0;

// Visual
pub const AERIALIST_RADIUS: f32 = 8.0 * UNIT_SCALE;
pub const AERIALIST_FLYING_FRAMES: usize = 2;

// Spawn tier (tier 2 = levels 11+)
pub const AERIALIST_START_TIER: u32 = 2;

// Base count at tier_level 1 when they first appear
pub const BASE_AERIALIST_COUNT: u32 = 2;
pub const AERIALISTS_PER_LEVEL: u32 = 1;
