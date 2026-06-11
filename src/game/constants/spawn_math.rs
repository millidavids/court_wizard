use super::positions::WIZARD_POSITION;

// ===== Spawn Grid Constants (Shared) =====

/// Angular spacing between columns (radians). ~0.1 rad ≈ 274 units at range 2736.
/// Used by both attacker and defender spawn grids.
pub const GRID_ANGULAR_SPACING: f32 = 0.1;

/// Radial depth of each row (distance between row centers).
/// Used by both attacker and defender spawn grids.
pub const GRID_ROW_DEPTH: f32 = 300.0;

// ===== Staging Points =====

/// Number of staging points around the wizard's range arc.
pub const STAGING_POINT_COUNT: usize = 7;

/// 7 staging points in an arc around the wizard's spell range.
/// Computed at radius ~2759 from the wizard position (-1400, 1700) in XZ,
/// centered on the original staging point angle (~-46.57 deg), spaced 10 deg apart.
/// Point 3 is the original staging point (500, -300).
pub const STAGING_POINTS: [(f32, f32); STAGING_POINT_COUNT] = [
    (1244.0, 913.0),  // +30 deg from center
    (1067.0, 466.0),  // +20 deg from center
    (818.0, 59.0),    // +10 deg from center
    (500.0, -300.0),  // center (original staging point)
    (117.0, -604.0),  // -10 deg from center
    (-307.0, -833.0), // -20 deg from center
    (-759.0, -983.0), // -30 deg from center
];

/// Index of the center staging point (original position).
/// Bosses and fallback logic always use this point.
pub const CENTER_STAGING_INDEX: usize = 3;

/// Satisfaction radius for the staging flow field (in pathfinding grid cells).
/// Units within this radius stop receiving flow directions and hold position.
/// 50 cells = 500 world units.
pub const STAGING_SATISFACTION_RADIUS: usize = 50;

/// Activation radius in world units. When 90% of a wave's living units
/// are within this distance of the staging point, the wave activates.
/// Must be >= satisfaction radius (in world units) so units that stop
/// at the edge of the flow field dead zone still count as arrived.
pub const STAGING_ACTIVATION_RADIUS: f32 = 600.0;

/// Fraction of a wave's living units that must be within the activation
/// radius before the wave activates. Dead units don't count against this.
pub const WAVE_ACTIVATION_THRESHOLD: f32 = 0.9;

/// Maximum seconds a wave can spend staging before force-activating.
/// Prevents the game from stalling if too many units get stuck.
pub const WAVE_STAGING_TIMEOUT: f32 = 15.0;

/// Time scale multiplier when no activated attackers are on the field.
/// Speeds up the march from spawn to staging area.
pub const STAGING_SPEEDUP: f64 = 5.0;

// ===== Attacker Tunnel Spawn Points =====

/// X coordinate for attacker spawn points (just beyond the right wall).
pub const ATTACKER_SPAWN_X: f32 = 3100.0;

/// Spawn depth offset for archers (behind infantry for formation ordering).
pub const ARCHER_SPAWN_DEPTH_OFFSET: f32 = 400.0;

/// Spawn depth offset for assassins (behind archers for formation ordering).
pub const ASSASSIN_SPAWN_DEPTH_OFFSET: f32 = 700.0;

/// The two static spawn points behind the right wall, aligned with tunnel archways.
/// Units split evenly between these and path through the tunnels to the staging area.
pub const ATTACKER_SPAWN_POINTS: [(f32, f32); 2] = [
    (ATTACKER_SPAWN_X, -375.0),  // Bottom tunnel
    (ATTACKER_SPAWN_X, -1575.0), // Top tunnel
];

/// Returns the spawn position for an attacker unit, alternating between the two tunnel spawn points.
/// `depth_offset` pushes the unit further behind the wall for formation ordering.
pub fn attacker_spawn_position(unit_index: u32, depth_offset: f32) -> (f32, f32) {
    let (x, z) = ATTACKER_SPAWN_POINTS[(unit_index % 2) as usize];
    (x + depth_offset, z)
}

// ===== Defender Spawn Grid Constants =====

/// Number of columns in the defender spawn grid.
pub const DEFENDER_GRID_COLS: u32 = 5;

/// Number of rows in the defender spawn grid.
pub const DEFENDER_GRID_ROWS: u32 = 4;

/// Center angle from wizard toward defender spawn area (radians).
pub const DEFENDER_GRID_CENTER_ANGLE: f32 = -0.70;

/// Ground-plane distance from wizard to defender spawn grid.
pub const DEFENDER_GRID_GROUND_RANGE: f32 = 200.0;

/// Returns the world-space center of the defender spawn grid.
/// Used as the rally point when defenders return between waves.
pub fn defender_spawn_center() -> (f32, f32) {
    calculate_defender_grid_position(DEFENDER_GRID_ROWS / 2, DEFENDER_GRID_COLS / 2)
}

/// Calculates the world position of a defender grid cell.
///
/// The defender grid is a radial arc positioned opposite from attackers,
/// closer to the battlefield center. Uses the same angular spacing and row depth
/// as the attacker grid for consistency.
///
/// Grid is flipped 180 degrees: row 0 is farthest from attackers (archers in back),
/// higher rows are closer to attackers (infantry in front).
///
/// # Arguments
/// * `row` - Row index (0 = farthest from attackers/closest to wizard, for archers)
/// * `col` - Column index (0-3 for 4 columns, centered around center angle)
///
/// # Returns
/// Tuple of (x, z) world coordinates for the cell center
pub fn calculate_defender_grid_position(row: u32, col: u32) -> (f32, f32) {
    let col_offset = col as f32 - (DEFENDER_GRID_COLS as f32 - 1.0) / 2.0; // Center columns
    let angle = DEFENDER_GRID_CENTER_ANGLE + col_offset * GRID_ANGULAR_SPACING;
    // Row 0 starts at base range, increasing rows go AWAY from wizard (same as attackers)
    let radius = DEFENDER_GRID_GROUND_RANGE + GRID_ROW_DEPTH / 2.0 + row as f32 * GRID_ROW_DEPTH;
    let x = WIZARD_POSITION.x + radius * angle.cos();
    let z = WIZARD_POSITION.z + radius * angle.sin();
    (x, z)
}
