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

/// Defender spawn points in a 2×2 grid under the castle.
/// Castle is at (-1550, 1550). These points form a formation directly below it.
/// Grid spacing: 300 units between points (reduced from 400 for tighter formation)
pub const DEFENDER_SPAWN_POINTS: [(f32, f32); 4] = [
    (-1700.0, 1200.0), // Southwest (was -1750, 1150)
    (-1400.0, 1200.0), // Southeast (was -1350, 1150)
    (-1700.0, 1500.0), // Northwest (was -1750, 1550)
    (-1400.0, 1500.0), // Northeast (was -1350, 1550)
];

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
pub const COHESION_STRENGTH: f32 = 0.1;

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

// ===== Formation Grid Constants =====

/// Starting position for attacker formations (back-right corner from camera view)
/// Grid expands forward (toward camera, +Z) and left (toward defenders, -X) as more groups are added
/// Positioned so front diagonal is just beyond wizard's spell range (3000 units from wizard)
/// Wizard is at (-1425, 1550)
pub const FORMATION_GRID_START_X: f32 = 1200.0;
pub const FORMATION_GRID_START_Z: f32 = 0.0;

/// Distance to push back each diagonal row to maintain consistent meeting point
/// As more diagonals are added, they spawn progressively further back
pub const DIAGONAL_PUSHBACK_DISTANCE: f32 = 400.0;

/// Spacing between formation groups in the grid
pub const FORMATION_GRID_SPACING: f32 = 300.0;

// ===== Level-Based Spawn Calculations =====

/// Calculates the number of infantry groups based on level.
/// Level 1: 3 groups
/// Every odd level (3, 5, 7, ...): add 1 group
pub const fn calculate_infantry_groups(level: u32) -> u32 {
    3 + (level - 1) / 2
}

/// Calculates the number of archer groups based on level.
/// Level 1-3: 1 group
/// Level 4+: 1 + level/4
pub const fn calculate_archer_groups(level: u32) -> u32 {
    1 + (level.saturating_sub(1)) / 4
}

/// Calculates the size of each group based on level.
/// Level 1: base size (10 infantry, 5 archers)
/// Every even level (2, 4, 6, ...): +1 to group size
pub const fn calculate_group_size_bonus(level: u32) -> u32 {
    (level - 1) / 2
}

/// Calculates a grid position for a formation group.
/// Groups are placed in a grid starting from back-right, expanding forward and left.
/// Archers are placed in the back rows.
///
/// # Arguments
/// * `group_index` - Index of the group (0-based)
/// * `total_groups` - Total number of groups (used to calculate global pushback)
///
/// # Returns
/// Tuple of (x, z) coordinates for the group's spawn point
pub fn calculate_formation_grid_position(group_index: u32, total_groups: u32) -> (f32, f32) {
    // Diagonal pyramid pattern starting from back-right corner
    // Position 0: (0, 0) - back-right corner
    // Positions 1-2: (1, 0), (0, 1) - diagonal 1
    // Positions 3-5: (2, 0), (1, 1), (0, 2) - diagonal 2
    // Positions 6-9: (3, 0), (2, 1), (1, 2), (0, 3) - diagonal 3
    // Each diagonal n has (n+1) elements

    // Find which diagonal this group_index falls on
    // Total elements up to diagonal n: (n+1)(n+2)/2
    let mut diagonal = 0u32;
    let mut cumulative = 0u32;
    loop {
        let next_cumulative = cumulative + diagonal + 1;
        if group_index < next_cumulative {
            break;
        }
        cumulative = next_cumulative;
        diagonal += 1;
    }

    // Position within this diagonal
    let position_in_diagonal = group_index - cumulative;

    // Find the maximum diagonal actually used (not the minimum diagonal that COULD fit)
    // We want the diagonal of the last group, not the theoretical minimum
    let mut last_diagonal = 0u32;
    let mut cumulative_for_max = 0u32;
    loop {
        let next_cumulative = cumulative_for_max + last_diagonal + 1;
        if next_cumulative >= total_groups {
            break;
        }
        cumulative_for_max = next_cumulative;
        last_diagonal += 1;
    }
    let max_diagonal = last_diagonal;

    // Push back ALL positions based on max_diagonal (not individual diagonal)
    // This ensures all groups move back together when a new diagonal is needed
    // Push toward back-right corner: more positive X (right) and more negative Z (back)
    let global_pushback = max_diagonal as f32 * DIAGONAL_PUSHBACK_DISTANCE;

    // On diagonal n, positions expand from back-right toward front-left (camera perspective)
    // position_in_diagonal = 0 -> n steps left, 0 steps forward
    // position_in_diagonal = k -> (n-k) steps left, k steps forward
    // Camera at (-1000, 2500, 2500), so forward = +Z, left = -X
    let steps_left = diagonal - position_in_diagonal;
    let steps_forward = position_in_diagonal;

    // Apply pushback diagonally toward back-right corner (positive X, negative Z)
    let x = FORMATION_GRID_START_X - (steps_left as f32 * FORMATION_GRID_SPACING) + global_pushback
        - 1300.0;
    let z = FORMATION_GRID_START_Z + (steps_forward as f32 * FORMATION_GRID_SPACING)
        - global_pushback
        - 100.0;

    (x, z)
}
