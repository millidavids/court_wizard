//! Shared constants for the game.
//!
//! Contains all hardcoded values that are used across multiple modules
//! to ensure consistency and make changes easier.

use bevy::prelude::*;

// ===== Battlefield Dimensions =====

/// Size of the battlefield (width and depth).
pub const BATTLEFIELD_SIZE: f32 = 6000.0;

// ===== Castle Positioning =====

/// Castle position in 3D space.
pub const CASTLE_POSITION: Vec3 = Vec3::new(-1550.0, 1200.0, 1550.0);

/// Castle rotation in degrees.
pub const CASTLE_ROTATION_DEGREES: f32 = 45.0;

/// Castle dimensions (width, depth).
pub const CASTLE_WIDTH: f32 = 300.0;
pub const CASTLE_DEPTH: f32 = 2000.0;

/// Wizard offset from castle position.
const WIZARD_OFFSET: Vec3 = Vec3::new(125.0, 30.0, 0.0);

// ===== Spawn Areas =====

/// Defender spawn area (closer to center for faster clash with attackers).
/// Spawn defenders to meet attackers near the center of the battlefield.
/// X range: -1000 to -500
/// Z range: 0 to 500
pub const DEFENDER_SPAWN_X_MIN: f32 = -1000.0;
pub const DEFENDER_SPAWN_Z_MIN: f32 = 0.0;

/// Attacker spawn area (closer to wizard for faster testing).
/// Spawn attackers within wizard's spell range for immediate testing.
/// X range: 500 to 1000
/// Z range: -500 to 0
pub const ATTACKER_SPAWN_X_MIN: f32 = 500.0;
pub const ATTACKER_SPAWN_Z_MIN: f32 = -500.0;

// ===== Unit Positioning =====

/// Wizard position in 3D space (on castle platform).
/// Calculated as castle position plus offset.
pub const WIZARD_POSITION: Vec3 = Vec3::new(
    CASTLE_POSITION.x + WIZARD_OFFSET.x,
    CASTLE_POSITION.y + WIZARD_OFFSET.y,
    CASTLE_POSITION.z + WIZARD_OFFSET.z,
);

// ===== Gameplay Constants =====

/// Initial number of defenders spawned at game start.
pub const INITIAL_DEFENDER_COUNT: u32 = 100;

/// Initial number of attackers spawned at game start.
pub const INITIAL_ATTACKER_COUNT: u32 = 100;

// ===== Unit Stats =====

/// Default health for all units.
pub const UNIT_HEALTH: f32 = 50.0;

/// Default movement speed for all units (units per second).
pub const UNIT_MOVEMENT_SPEED: f32 = 100.0;

/// Hitbox height for defender units.
pub const DEFENDER_HITBOX_HEIGHT: f32 = 25.0;

/// Hitbox height for attacker units.
pub const ATTACKER_HITBOX_HEIGHT: f32 = 20.0;

// ===== Spawn Distribution =====

/// Offset multiplier for distributing spawned units in a pattern.
pub const SPAWN_OFFSET_MULTIPLIER: f32 = 0.31415;

/// Radius of the spawn distribution area (units spawn within this radius).
pub const SPAWN_DISTRIBUTION_RADIUS: f32 = 50.0;

// ===== Movement Constants =====

/// Velocity damping coefficient (reduces velocity each frame to prevent excessive momentum).
pub const VELOCITY_DAMPING: f32 = 0.85;

/// Steering force strength for acceleration-based movement.
pub const STEERING_FORCE: f32 = 500.0;

/// Movement speed multiplier when in melee combat (slows units down to prevent running around).
pub const MELEE_SLOWDOWN_FACTOR: f32 = 0.3;

/// Distance threshold to be considered "in melee" for slowdown purposes.
pub const MELEE_SLOWDOWN_DISTANCE: f32 = 50.0;

/// Approximate frame time for attack window detection (in seconds).
pub const APPROX_FRAME_TIME: f32 = 0.016;

// ===== Flocking Constants =====

/// Maximum distance to consider a unit as a neighbor for flocking behavior.
pub const NEIGHBOR_DISTANCE: f32 = 100.0;

/// Distance threshold for separation force to apply.
/// Units only separate when they're very close to colliding (just beyond hitbox radius).
pub const SEPARATION_DISTANCE: f32 = 5.0;

/// Strength of the separation force (pushes units apart).
/// Since we're using normalized directions, this should be small (0-1 range).
pub const SEPARATION_STRENGTH: f32 = 0.05;

/// Strength of the alignment force (matches neighbor velocities).
/// Since we're using normalized directions, this should be small (0-1 range).
pub const ALIGNMENT_STRENGTH: f32 = 0.1;

/// Strength of the cohesion force (pulls units toward group center). Set to 0.0 to disable.
pub const COHESION_STRENGTH: f32 = 0.2;

/// Maximum allowed overlap between hitboxes as a percentage. 0.0 = no overlap allowed.
pub const MAX_OVERLAP_PERCENT: f32 = 0.0;

/// Minimum distance threshold for collision calculations (avoids division by zero).
pub const MIN_DISTANCE_THRESHOLD: f32 = 0.01;

/// Collision resolution iterations (higher = more accurate but more expensive).
pub const COLLISION_ITERATIONS: u32 = 4;

// ===== Targeting Constants =====

// ===== Combat Constants =====

/// Attack range multiplier based on hitbox radius.
pub const ATTACK_RANGE_MULTIPLIER: f32 = 1.5;

/// Damage dealt per attack.
pub const ATTACK_DAMAGE: f32 = 10.0;

/// Duration of one complete attack cycle in seconds.
pub const ATTACK_CYCLE_DURATION: f32 = 2.0;

// ===== Effectiveness System =====

/// Bonus to effectiveness per ally in melee range (+10% each).
pub const EFFECTIVENESS_ALLY_BONUS_PER_UNIT: f32 = 0.10;

/// Penalty to effectiveness per enemy in melee range (-15% each).
pub const EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT: f32 = -0.15;

/// Minimum effectiveness cap (10% of base).
pub const EFFECTIVENESS_MIN: f32 = 0.1;

/// Maximum effectiveness cap (200% of base).
pub const EFFECTIVENESS_MAX: f32 = 2.0;
