use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_PURPLE, tint};

// Visual appearance
pub const BRUTE_COLOR: Color = tint(ATTACKER_BASE, TINT_PURPLE, 0.2); // Purple tint
pub const BRUTE_ELLIPSE_WIDTH: f32 = 20.0; // Ellipse width (X axis)
pub const BRUTE_ELLIPSE_DEPTH: f32 = 30.0; // Ellipse depth (Z axis) - longer oval
pub const BRUTE_RADIUS: f32 = 20.0; // Horizontal radius for collision detection
pub const BRUTE_HITBOX_HEIGHT: f32 = 60.0; // Vertical height

// Movement
pub const BRUTE_MOVEMENT_SPEED: f32 = 75.0; // Slower than infantry (100.0) for tank-like feel

// Combat
pub const BRUTE_HEALTH: f32 = 500.0; // 4x normal unit health (50.0 * 4)
pub const BRUTE_AOE_DAMAGE: f32 = 200.0; // AOE splash damage
pub const BRUTE_AOE_RADIUS: f32 = 30.0; // AOE effect radius

// Spawn tier
pub const BRUTE_START_TIER: u32 = 4;
