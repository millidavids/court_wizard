//! Shared constants for the game.
//!
//! Contains all hardcoded values that are used across multiple modules
//! to ensure consistency and make changes easier.

use bevy::prelude::*;

// ===== Battlefield Dimensions =====

/// Size of the battlefield (width and depth).
pub const BATTLEFIELD_SIZE: f32 = 6000.0;

/// Half size of the battlefield (for calculating edges).
pub const BATTLEFIELD_HALF_SIZE: f32 = BATTLEFIELD_SIZE / 2.0;

// ===== Castle Positioning =====

/// Castle position in 3D space.
pub const CASTLE_POSITION: Vec3 = Vec3::new(-1300.0, 1200.0, 1300.0);

/// Castle rotation in degrees.
pub const CASTLE_ROTATION_DEGREES: f32 = 45.0;

/// Castle dimensions (width, depth).
pub const CASTLE_WIDTH: f32 = 300.0;
pub const CASTLE_DEPTH: f32 = 2000.0;

// ===== Spawn Areas =====

/// Size of the spawn area at battlefield edges (in pixels).
pub const EDGE_SPAWN_AREA_SIZE: f32 = 200.0;

/// Defender spawn area (bottom-left edge of battlefield).
/// X range: -BATTLEFIELD_HALF_SIZE to -BATTLEFIELD_HALF_SIZE + EDGE_SPAWN_AREA_SIZE
/// Z range: BATTLEFIELD_HALF_SIZE - EDGE_SPAWN_AREA_SIZE to BATTLEFIELD_HALF_SIZE
pub const DEFENDER_SPAWN_X_MIN: f32 = -BATTLEFIELD_HALF_SIZE;
pub const DEFENDER_SPAWN_Z_MIN: f32 = BATTLEFIELD_HALF_SIZE - EDGE_SPAWN_AREA_SIZE;

/// Attacker spawn area (top-right edge of battlefield).
/// X range: BATTLEFIELD_HALF_SIZE - EDGE_SPAWN_AREA_SIZE to BATTLEFIELD_HALF_SIZE
/// Z range: -BATTLEFIELD_HALF_SIZE to -BATTLEFIELD_HALF_SIZE + EDGE_SPAWN_AREA_SIZE
pub const ATTACKER_SPAWN_X_MIN: f32 = BATTLEFIELD_HALF_SIZE - EDGE_SPAWN_AREA_SIZE;
pub const ATTACKER_SPAWN_Z_MIN: f32 = -BATTLEFIELD_HALF_SIZE;

// ===== Unit Positioning =====

/// Y position for units moving on the battlefield.
pub const UNIT_Y_POSITION: f32 = 50.0;

/// Wizard position in 3D space (on castle platform).
pub const WIZARD_POSITION: Vec3 = Vec3::new(-1150.0, 1230.0, 1400.0);

// ===== Gameplay Constants =====

/// Distance at which defenders activate and start moving.
/// Based on battlefield size with a multiplier.
pub const DEFENDER_ACTIVATION_DISTANCE: f32 = BATTLEFIELD_SIZE * (8000.0 / 6000.0);

/// Initial number of defenders spawned at game start.
pub const INITIAL_DEFENDER_COUNT: u32 = 100;

/// Initial number of attackers spawned at game start.
pub const INITIAL_ATTACKER_COUNT: u32 = 100;

// ===== Combat Constants =====

/// Attack range multiplier based on hitbox radius.
pub const ATTACK_RANGE_MULTIPLIER: f32 = 1.5;

/// Damage dealt per attack.
pub const ATTACK_DAMAGE: f32 = 10.0;
