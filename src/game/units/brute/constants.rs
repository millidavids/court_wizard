use crate::game::constants::UNIT_SCALE;

// Visual appearance
/// Scale multiplier for brute (larger than normal infantry).
pub const BRUTE_SCALE: f32 = 2.5;
pub const BRUTE_RADIUS: f32 = 20.0 * UNIT_SCALE; // Horizontal radius for collision detection
pub const BRUTE_HITBOX_HEIGHT: f32 = 60.0 * UNIT_SCALE; // Vertical height

// Movement
pub const BRUTE_MOVEMENT_SPEED: f32 = 115.0;

// Combat
pub const BRUTE_HEALTH: f32 = 1000.0; // 4x normal unit health doubled

// Spawn tier
pub const BRUTE_START_TIER: u32 = 2;
