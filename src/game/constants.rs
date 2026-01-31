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

/// Number of columns in the attacker spawn grid.
pub const GRID_COLS: u32 = 6;

/// Number of rows in the attacker spawn grid.
pub const GRID_ROWS: u32 = 6;

/// Center angle from wizard toward spawn area (radians).
/// atan2(0 - 1550, 1200 - (-1425)) ≈ -0.53 rad
pub const GRID_CENTER_ANGLE: f32 = -0.70;

/// Angular spacing between columns (radians). ~0.1 rad ≈ 274 units at range 2736.
pub const GRID_ANGULAR_SPACING: f32 = 0.1;

/// Radial depth of each row (distance between row centers).
pub const GRID_ROW_DEPTH: f32 = 300.0;

/// Ground-plane spell range: sqrt(3000² - 1230²) ≈ 2736.
pub const GRID_GROUND_RANGE: f32 = 3236.0;

// ===== Level-Based Spawn Calculations =====

/// Maximum units per grid cell before spilling to the next cell.
pub const MAX_UNITS_PER_CELL: u32 = 10;

/// Base infantry count at level 1.
pub const BASE_INFANTRY_COUNT: u32 = 60;

/// Infantry added per level after level 1.
pub const INFANTRY_PER_LEVEL: u32 = 5;

/// Base archer count at level 1.
pub const BASE_ARCHER_COUNT: u32 = 10;

/// Archers added per level after level 1.
pub const ARCHERS_PER_LEVEL: u32 = 2;

/// Calculates total infantry for a given level.
pub const fn calculate_total_infantry(level: u32) -> u32 {
    BASE_INFANTRY_COUNT + (level - 1) * INFANTRY_PER_LEVEL
}

/// Calculates total archers for a given level.
pub const fn calculate_total_archers(level: u32) -> u32 {
    BASE_ARCHER_COUNT + (level - 1) * ARCHERS_PER_LEVEL
}

/// Calculates the number of cells needed for a unit count (ceil division by MAX_UNITS_PER_CELL).
pub const fn cells_needed(total_units: u32) -> u32 {
    total_units.div_ceil(MAX_UNITS_PER_CELL)
}

/// Returns a Vec of unit counts per cell, distributing units evenly.
/// Each cell gets up to MAX_UNITS_PER_CELL, with remainder spread across first cells.
pub fn distribute_units_to_cells(total_units: u32) -> Vec<u32> {
    let num_cells = cells_needed(total_units);
    if num_cells == 0 {
        return vec![];
    }
    let base_per_cell = total_units / num_cells;
    let remainder = total_units % num_cells;
    (0..num_cells)
        .map(|i| {
            if i < remainder {
                base_per_cell + 1
            } else {
                base_per_cell
            }
        })
        .collect()
}

/// Calculates the world position of a grid cell.
///
/// The grid is a 6x6 radial arc centered on the wizard's spell range ring.
/// Row 0 is closest to the wizard (near edge tangent to range ring).
/// Columns fan out angularly around the center angle.
///
/// # Arguments
/// * `row` - Row index (0 = closest to wizard)
/// * `col` - Column index (0-5, centered around center angle)
///
/// # Returns
/// Tuple of (x, z) world coordinates for the cell center
pub fn calculate_grid_cell_position(row: u32, col: u32) -> (f32, f32) {
    let col_offset = col as f32 - 2.5; // centers 6 columns: -2.5 .. 2.5
    let angle = GRID_CENTER_ANGLE + col_offset * GRID_ANGULAR_SPACING;
    let radius = GRID_GROUND_RANGE + GRID_ROW_DEPTH / 2.0 + row as f32 * GRID_ROW_DEPTH;
    let x = WIZARD_POSITION.x + radius * angle.cos();
    let z = WIZARD_POSITION.z + radius * angle.sin();
    (x, z)
}

/// Computes Dijkstra distance for a cell from the bottom-center of the grid.
/// Distance = row + |col - 2.5| rounded: min(|col - 2|, |col - 3|) + row
fn grid_cell_distance(row: u32, col: u32) -> u32 {
    let col_dist = if col <= 2 { 2 - col } else { col - 3 };
    row + col_dist
}

/// Generates the ordered list of cells for infantry and archer spawns.
///
/// Infantry cells are sorted by Dijkstra distance from bottom-center (row 0, cols 2-3).
/// Archers fill the row directly behind the last infantry row, in middle-out column order.
///
/// # Returns
/// (infantry_cells, archer_cells) - each is a Vec of (row, col) tuples
pub fn calculate_spawn_cells(
    num_infantry_cells: u32,
    num_archer_cells: u32,
) -> (Vec<(u32, u32)>, Vec<(u32, u32)>) {
    // Build all cells sorted by distance, then by column proximity to center
    let mut all_cells: Vec<(u32, u32, u32)> = Vec::new(); // (distance, row, col)
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            all_cells.push((grid_cell_distance(row, col), row, col));
        }
    }
    // Sort by distance, then by row (closer first), then by column proximity to center
    all_cells.sort_by(|a, b| {
        a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then({
            // Tie-break: columns closer to center first
            let a_col_dist = if a.2 <= 2 { 2 - a.2 } else { a.2 - 3 };
            let b_col_dist = if b.2 <= 2 { 2 - b.2 } else { b.2 - 3 };
            a_col_dist.cmp(&b_col_dist)
        })
    });

    // Take infantry cells
    let infantry_count = (num_infantry_cells as usize).min(all_cells.len());
    let infantry_cells: Vec<(u32, u32)> = all_cells[..infantry_count]
        .iter()
        .map(|&(_, r, c)| (r, c))
        .collect();

    // Find the last infantry row
    let last_infantry_row = infantry_cells.iter().map(|&(r, _)| r).max().unwrap_or(0);

    // Archers go in the row directly behind the last infantry row
    let archer_row = last_infantry_row + 1;
    let col_fill_order: [u32; 6] = [2, 3, 1, 4, 0, 5];
    let archer_cells: Vec<(u32, u32)> = col_fill_order
        .iter()
        .take(num_archer_cells as usize)
        .map(|&col| (archer_row, col))
        .collect();

    (infantry_cells, archer_cells)
}
