//! Shared constants for the game.
//!
//! Contains all hardcoded values that are used across multiple modules
//! to ensure consistency and make changes easier.

use bevy::prelude::*;

// ===== Unit Scale =====

/// Global scale factor for all unit sizes (sprite dimensions, hitbox radii, and size-dependent distances).
pub const UNIT_SCALE: f32 = 4.0;

// ===== Team Base Colors =====

/// Base color for defender units (light gray).
pub const DEFENDER_BASE: Color = Color::srgb(0.65, 0.65, 0.65);

/// Base color for attacker units (dark gray).
pub const ATTACKER_BASE: Color = Color::srgb(0.3, 0.3, 0.3);

/// Blends a tint color into a base color at the given strength (0.0 = pure base, 1.0 = pure tint).
pub const fn tint(base: Color, tint_color: Color, strength: f32) -> Color {
    let Color::Srgba(b) = base else {
        return base;
    };
    let Color::Srgba(t) = tint_color else {
        return base;
    };
    Color::Srgba(bevy::color::Srgba {
        red: b.red + (t.red - b.red) * strength,
        green: b.green + (t.green - b.green) * strength,
        blue: b.blue + (t.blue - b.blue) * strength,
        alpha: b.alpha,
    })
}

/// Dims a color toward gray and applies an alpha value.
/// Used for corpse materials — tints toward gray and makes semi-transparent.
pub const fn dim(base: Color, gray_strength: f32, alpha: f32) -> Color {
    let Color::Srgba(b) = base else {
        return base;
    };
    let gray = 0.5;
    Color::Srgba(bevy::color::Srgba {
        red: b.red + (gray - b.red) * gray_strength,
        green: b.green + (gray - b.green) * gray_strength,
        blue: b.blue + (gray - b.blue) * gray_strength,
        alpha,
    })
}

// ===== Common Tint Colors =====

pub const TINT_RED: Color = Color::srgb(1.0, 0.2, 0.2);
pub const TINT_ORANGE: Color = Color::srgb(1.0, 0.6, 0.0);
pub const TINT_PURPLE: Color = Color::srgb(0.5, 0.1, 0.8);
// ===== Undead Color =====

/// Base color for undead units (purple).
pub const UNDEAD_BASE: Color = Color::srgb(0.5, 0.2, 0.7);

// ===== Corpse Colors (tinted red then dimmed) =====

/// Alpha for corpse materials — semi-transparent so alive units show through.
pub const CORPSE_ALPHA: f32 = 0.4;

/// Corpse color for defender units.
pub const DEFENDER_CORPSE_COLOR: Color = dim(tint(DEFENDER_BASE, TINT_RED, 0.4), 0.3, CORPSE_ALPHA);
/// Corpse color for attacker units.
pub const ATTACKER_CORPSE_COLOR: Color = dim(tint(ATTACKER_BASE, TINT_RED, 0.4), 0.3, CORPSE_ALPHA);
/// Corpse color for undead units.
pub const UNDEAD_CORPSE_COLOR: Color = dim(tint(UNDEAD_BASE, TINT_RED, 0.4), 0.3, CORPSE_ALPHA);
/// Corpse color for the king.
pub const KING_CORPSE_COLOR: Color = dim(
    tint(Color::srgb(0.65, 0.65, 0.9), TINT_RED, 0.4),
    0.3,
    CORPSE_ALPHA,
);

// ===== Earth/Stone Colors =====

/// Dark reddish-brown stone color used for stone underlays, wall sides, trampling,
/// and dirt borders. The canonical "earth" color throughout the battlefield.
pub const STONE_COLOR_DARK: Color = Color::srgb(0.11, 0.04, 0.03);

/// Light reddish-brown stone color, paired with STONE_COLOR_DARK for noise blending
/// on stone surfaces, wall of stone sides, and dirt border accents.
pub const STONE_COLOR_LIGHT: Color = Color::srgb(0.20, 0.10, 0.07);

// ===== Battlefield Dimensions =====

/// Size of the battlefield (width and depth).
pub const BATTLEFIELD_SIZE: f32 = 6000.0;

/// Extra world units added to +X side of pathfinding grid for spawn area behind right wall.
pub const PATHFINDING_X_EXTENSION: f32 = 2000.0;

// ===== Castle Positioning =====

/// Castle position in 3D space (shifted back toward camera along the 45° diagonal).
pub const CASTLE_POSITION: Vec3 = Vec3::new(-1700.0, 1200.0, 1700.0);

/// Castle rotation in degrees.
pub const CASTLE_ROTATION_DEGREES: f32 = 35.0;

/// Castle width — used for the castle wall plane.
pub const CASTLE_WIDTH: f32 = 300.0;

/// Wizard offset from castle position.
/// Y offset = half sprite height so the wizard's feet rest on the castle platform.
pub(crate) const WIZARD_OFFSET: Vec3 = Vec3::new(300.0, 450.0, 0.0);

// ===== Unit Positioning =====

/// Wizard position in 3D space (on castle platform).
/// Calculated as castle position plus offset.
pub const WIZARD_POSITION: Vec3 = Vec3::new(
    CASTLE_POSITION.x + WIZARD_OFFSET.x,
    CASTLE_POSITION.y + WIZARD_OFFSET.y,
    CASTLE_POSITION.z + WIZARD_OFFSET.z,
);

/// Offset from wizard position to place the cauldron beside the wizard on the castle wall.
const SPELL_OFFSET: Vec3 = Vec3::new(100.0, 90.0, 30.0);

/// Spell origin point — where projectiles and beams originate from.
/// Same as wizard position since the wizard doesn't move.
pub const SPELL_ORIGIN: Vec3 = Vec3::new(
    WIZARD_POSITION.x + SPELL_OFFSET.x,
    WIZARD_POSITION.y + SPELL_OFFSET.y,
    WIZARD_POSITION.z + SPELL_OFFSET.z,
);

// ===== Castle 2 Positioning (Multiplayer — opposite corner) =====

/// Castle 2 position in 3D space (diagonally opposite from Castle 1).
pub const CASTLE_2_POSITION: Vec3 = Vec3::new(1550.0, 1200.0, -1550.0);

/// Castle 2 rotation in degrees (facing opposite direction).
pub const CASTLE_2_ROTATION_DEGREES: f32 = CASTLE_ROTATION_DEGREES + 180.0;

/// Wizard 2 position (on Castle 2 platform).
pub const WIZARD_2_POSITION: Vec3 = Vec3::new(
    CASTLE_2_POSITION.x - WIZARD_OFFSET.x,
    CASTLE_2_POSITION.y + WIZARD_OFFSET.y,
    CASTLE_2_POSITION.z - WIZARD_OFFSET.z,
);

// ===== Multiplayer Constants =====

/// Fixed infantry count for each side in multiplayer (no level scaling).
pub const MP_INFANTRY_COUNT: u32 = 60;

/// Fixed archer count for each side in multiplayer.
pub const MP_ARCHER_COUNT: u32 = 10;

/// Ground-plane distance from wizard to defender spawn grid in multiplayer.
/// Much closer than single-player so units spawn right against the castles.
pub const MP_DEFENDER_GRID_GROUND_RANGE: f32 = 100.0;

/// Calculates the world position of a defender grid cell for MP host side (Castle 1).
pub fn calculate_mp_defender_grid_position(row: u32, col: u32) -> (f32, f32) {
    let col_offset = col as f32 - (DEFENDER_GRID_COLS as f32 - 1.0) / 2.0;
    let angle = DEFENDER_GRID_CENTER_ANGLE + col_offset * GRID_ANGULAR_SPACING;
    let radius = MP_DEFENDER_GRID_GROUND_RANGE + GRID_ROW_DEPTH / 2.0 + row as f32 * GRID_ROW_DEPTH;
    let x = WIZARD_POSITION.x + radius * angle.cos();
    let z = WIZARD_POSITION.z + radius * angle.sin();
    (x, z)
}

/// Calculates the world position of a guest defender grid cell for MP (Castle 2).
pub fn calculate_mp_guest_defender_grid_position(row: u32, col: u32) -> (f32, f32) {
    let col_offset = col as f32 - (DEFENDER_GRID_COLS as f32 - 1.0) / 2.0;
    let mirrored_angle = DEFENDER_GRID_CENTER_ANGLE + std::f32::consts::PI;
    let angle = mirrored_angle + col_offset * GRID_ANGULAR_SPACING;
    let radius = MP_DEFENDER_GRID_GROUND_RANGE + GRID_ROW_DEPTH / 2.0 + row as f32 * GRID_ROW_DEPTH;
    let x = WIZARD_2_POSITION.x + radius * angle.cos();
    let z = WIZARD_2_POSITION.z + radius * angle.sin();
    (x, z)
}

// ===== Gameplay Constants =====

/// Initial number of defenders spawned at game start.
pub const INITIAL_DEFENDER_COUNT: u32 = 100;

/// Number of King's Guard units spawned at game start.
pub const KINGS_GUARD_COUNT: u32 = 10;

/// Radius of the orbit circle for King's Guard around the King.
pub const KINGS_GUARD_ORBIT_RADIUS: f32 = 20.0 * UNIT_SCALE;

// ===== Unit Stats =====

/// Default health for all units.
pub const UNIT_HEALTH: f32 = 100.0;

/// Default movement speed for all units (units per second).
pub const UNIT_MOVEMENT_SPEED: f32 = 100.0;

/// Global multiplier applied to all unit movement speeds.
/// 1.0 = normal speed, 0.8 = 20% slower, etc.
pub const GLOBAL_SPEED_MULTIPLIER: f32 = 0.55;

/// Hitbox height for defender units.
pub const DEFENDER_HITBOX_HEIGHT: f32 = 25.0 * UNIT_SCALE;

/// Hitbox height for attacker units.
pub const ATTACKER_HITBOX_HEIGHT: f32 = 20.0 * UNIT_SCALE;

// ===== Movement Constants =====

/// Velocity damping coefficient (reduces velocity each frame to prevent excessive momentum).
pub const VELOCITY_DAMPING: f32 = 0.85;

/// Steering force strength for acceleration-based movement.
pub const STEERING_FORCE: f32 = 500.0;

/// Movement speed multiplier when in melee combat (slows units down to prevent running around).
pub const MELEE_SLOWDOWN_FACTOR: f32 = 0.3;

/// Distance threshold to be considered "in melee" for slowdown purposes.
pub const MELEE_SLOWDOWN_DISTANCE: f32 = 50.0 * UNIT_SCALE;

/// Approximate frame time for attack window detection (in seconds).
pub const APPROX_FRAME_TIME: f32 = 0.016;

// ===== Flocking Constants =====

/// Maximum distance to consider a unit as a neighbor for flocking behavior.
pub const NEIGHBOR_DISTANCE: f32 = 100.0;

/// Distance threshold for separation force to apply.
/// Units only separate when they're very close to colliding (just beyond hitbox radius).
pub const SEPARATION_DISTANCE: f32 = 3.0 * UNIT_SCALE;

/// Strength of the separation force (pushes units apart).
/// Since we're using normalized directions, this should be small (0-1 range).
pub const SEPARATION_STRENGTH: f32 = 0.05;

/// Strength of the alignment force (matches neighbor velocities).
/// Since we're using normalized directions, this should be small (0-1 range).
pub const ALIGNMENT_STRENGTH: f32 = 0.1;

/// Strength of the cohesion force (pulls units toward group center). Set to 0.0 to disable.
pub const COHESION_STRENGTH: f32 = 0.1;

/// Maximum allowed overlap between hitboxes as a percentage. 0.0 = no overlap allowed.
pub const MAX_OVERLAP_PERCENT: f32 = 0.2;

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

// ===== Spawn Grid Constants (Shared) =====

/// Angular spacing between columns (radians). ~0.1 rad ≈ 274 units at range 2736.
/// Used by both attacker and defender spawn grids.
pub const GRID_ANGULAR_SPACING: f32 = 0.1;

/// Radial depth of each row (distance between row centers).
/// Used by both attacker and defender spawn grids.
pub const GRID_ROW_DEPTH: f32 = 300.0;

// ===== Attacker Spawn Grid Constants =====

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

/// Minimum number of staging points used per wave.
pub const MIN_WAVE_STAGING_POINTS: usize = 2;

/// Maximum number of staging points used per wave.
pub const MAX_WAVE_STAGING_POINTS: usize = 4;

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

// ===== Defender Activation =====

/// Distance at which defenders activate (XZ plane distance).
/// Once any enemy is within this range of any defender, all defenders activate.
pub const DEFENDER_ACTIVATION_RANGE: f32 = 800.0;

// ===== King's Retreat =====

/// Percentage of the wizard's spell range at which the King triggers a retreat.
/// At 90% of spell range, the King is dangerously close to the edge.
pub const RETREAT_TRIGGER_DISTANCE_PERCENT: f32 = 0.90;

/// Duration of retreat in seconds — defenders disengage and fall back to spawn.
pub const RETREAT_DURATION_SECS: f32 = 15.0;

// ===== Wave System =====

/// Seconds between each wave of attackers.
pub const WAVE_INTERVAL_SECONDS: f32 = 60.0;

/// Base number of waves at tier 0.
pub const BASE_WAVE_COUNT: u32 = 1;

/// Maximum number of waves per level (in endless mode tiers keep rising).
pub const MAX_WAVE_COUNT: u32 = 6;

/// Returns the number of waves for a given level.
/// Most boss levels have 1 wave (boss only). Lich tier (1) uses normal wave count
/// since the Lich spawns mid-game alongside regular waves.
/// Normal levels get `BASE_WAVE_COUNT + tier`, capped at `MAX_WAVE_COUNT`.
pub const fn calculate_wave_count(level: u32) -> u32 {
    if is_boss_level(level) && !is_lich_level(level) {
        1
    } else {
        let waves = BASE_WAVE_COUNT + get_tier(level);
        if waves > MAX_WAVE_COUNT {
            MAX_WAVE_COUNT
        } else {
            waves
        }
    }
}

// ===== Tier Progression =====

/// Number of levels per tier. Every 5 levels = one tier.
/// Level 5, 10, 15, 20... are boss-only levels.
pub const LEVELS_PER_TIER: u32 = 5;

/// Returns the tier for a given level (0-indexed).
/// Tier 0 = levels 1-5, Tier 1 = levels 6-10, etc.
pub const fn get_tier(level: u32) -> u32 {
    (level - 1) / LEVELS_PER_TIER
}

/// Returns the tier-local level (1-based, cycles 1-4 within each tier).
/// The 5th level of each tier is a boss level.
pub const fn get_tier_level(level: u32) -> u32 {
    (level - 1) % LEVELS_PER_TIER + 1
}

/// Returns true if the given level is a boss-only level.
/// Boss levels are every 5th level starting at level 5.
pub const fn is_boss_level(level: u32) -> bool {
    level >= LEVELS_PER_TIER && level.is_multiple_of(LEVELS_PER_TIER)
}

/// Returns true if this is a Lich boss level (tier 1, 5, 9, ... in the 4-boss cycle).
/// Lich levels use normal waves with the Lich spawning mid-game after all waves.
pub const fn is_lich_level(level: u32) -> bool {
    is_boss_level(level) && get_tier(level) % BOSS_CYCLE_LENGTH == 1
}

/// Number of unique boss types in the rotation cycle.
pub const BOSS_CYCLE_LENGTH: u32 = 4;

/// Returns the boss name for a given boss level, or None if not a boss level.
pub fn boss_name_for_level(level: u32) -> Option<&'static str> {
    if !is_boss_level(level) {
        return None;
    }
    // Cycle through bosses: Ogre (0), Lich (1), Hags (2), Dark Mage (3), repeat
    Some(match get_tier(level) % BOSS_CYCLE_LENGTH {
        0 => "Ogre",
        1 => "The Lich",
        2 => "Hags",
        3 => "Dark Mage",
        _ => unreachable!(),
    })
}

// ===== Endless Mode Scaling =====

/// The last tier that introduces new unit types (tier 4, levels 21-25).
pub const FINAL_INTRODUCTION_TIER: u32 = 4;

/// The last level in the introduction tiers (after which endless scaling begins).
pub const LAST_INTRODUCTION_LEVEL: u32 = (FINAL_INTRODUCTION_TIER + 1) * LEVELS_PER_TIER;

/// Extra infantry per level past the final introduction tier in Endless mode.
pub const ENDLESS_EXTRA_INFANTRY_PER_LEVEL: u32 = 3;

/// Extra archers per level past the final introduction tier in Endless mode.
pub const ENDLESS_EXTRA_ARCHERS_PER_LEVEL: u32 = 1;

/// Cumulative effectiveness boost per level past the final introduction tier (2% per level).
pub const ENDLESS_SCALING_PER_LEVEL: f32 = 0.02;

/// Returns the number of levels past the introduction tiers, or 0 if still in them.
fn levels_past_introduction(level: u32) -> u32 {
    level.saturating_sub(LAST_INTRODUCTION_LEVEL)
}

/// Returns the cumulative effectiveness bonus for endless scaling.
/// Returns 0.0 for levels within the introduction tiers.
pub fn endless_effectiveness_bonus(level: u32) -> f32 {
    levels_past_introduction(level) as f32 * ENDLESS_SCALING_PER_LEVEL
}

/// Returns extra infantry to add in Endless mode past the final introduction tier.
pub fn endless_extra_infantry(level: u32) -> u32 {
    levels_past_introduction(level) * ENDLESS_EXTRA_INFANTRY_PER_LEVEL
}

/// Returns extra archers to add in Endless mode past the final introduction tier.
pub fn endless_extra_archers(level: u32) -> u32 {
    levels_past_introduction(level) * ENDLESS_EXTRA_ARCHERS_PER_LEVEL
}

// ===== Level-Based Spawn Calculations =====

/// Maximum units per grid cell before spilling to the next cell.
pub const MAX_UNITS_PER_CELL: u32 = 10;

/// Base infantry count at tier_level 1.
pub const BASE_INFANTRY_COUNT: u32 = 60;

/// Infantry added per tier_level after tier_level 1.
pub const INFANTRY_PER_LEVEL: u32 = 5;

/// Base archer count at tier_level 1.
pub const BASE_ARCHER_COUNT: u32 = 10;

/// Archers added per tier_level after tier_level 1.
pub const ARCHERS_PER_LEVEL: u32 = 2;

/// Calculates total infantry for a given level using tier-based progression.
/// Unit counts reset each tier, scaling with tier_level (1-4).
pub const fn calculate_total_infantry(level: u32) -> u32 {
    let tier_level = get_tier_level(level);
    BASE_INFANTRY_COUNT + (tier_level - 1) * INFANTRY_PER_LEVEL
}

/// Calculates total archers for a given level using tier-based progression.
/// Unit counts reset each tier, scaling with tier_level (1-4).
pub const fn calculate_total_archers(level: u32) -> u32 {
    let tier_level = get_tier_level(level);
    BASE_ARCHER_COUNT + (tier_level - 1) * ARCHERS_PER_LEVEL
}

/// Base assassin count at tier_level 1 (when they first appear in tier 2).
pub const BASE_ASSASSIN_COUNT: u32 = 8;

/// Assassins added per tier_level after tier_level 1.
pub const ASSASSINS_PER_LEVEL: u32 = 2;

/// The tier at which assassins start spawning (tier 2, level 6+).
pub const ASSASSIN_START_TIER: u32 = 1;

/// Calculates total assassins for a given level.
/// Assassins only spawn from tier 2 onward, scaling similarly to archers.
pub const fn calculate_total_assassins(level: u32) -> u32 {
    if get_tier(level) < ASSASSIN_START_TIER {
        return 0;
    }
    let tier_level = get_tier_level(level);
    BASE_ASSASSIN_COUNT + (tier_level - 1) * ASSASSINS_PER_LEVEL
}

/// Calculates total aerialists for a given level.
/// Aerialists spawn from tier 2 onward (level 11+).
pub const fn calculate_total_aerialists(level: u32) -> u32 {
    use crate::game::units::aerialist::constants::{
        AERIALIST_START_TIER, AERIALISTS_PER_LEVEL, BASE_AERIALIST_COUNT,
    };
    if get_tier(level) < AERIALIST_START_TIER {
        return 0;
    }
    let tier_level = get_tier_level(level);
    BASE_AERIALIST_COUNT + (tier_level - 1) * AERIALISTS_PER_LEVEL
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
