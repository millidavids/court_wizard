//! Shared constants for the game.
//!
//! Contains all hardcoded values that are used across multiple modules
//! to ensure consistency and make changes easier.

mod colors;
mod positions;
mod spawn_math;
mod tuning;
mod wave_tiers;

// Re-export all public items from each concern module.
// Every item that was pub/pub(crate) in the original constants.rs is re-exported
// here so all call sites at crate::game::constants::X continue to resolve.

pub use colors::{
    ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, KING_CORPSE_COLOR, STONE_COLOR_DARK,
    STONE_COLOR_LIGHT, UNDEAD_CORPSE_COLOR, UNIT_SCALE, tint,
};

pub use positions::{
    BATTLEFIELD_SIZE, CASTLE_2_POSITION, CASTLE_2_ROTATION_DEGREES, CASTLE_POSITION,
    CASTLE_ROTATION_DEGREES, CASTLE_WIDTH, MP_ARCHER_COUNT, MP_DEFENDER_GRID_GROUND_RANGE,
    MP_INFANTRY_COUNT, MP_TERRAIN_LEVEL, PATHFINDING_X_EXTENSION, SPELL_2_ORIGIN,
    SPELL_COOP_ORIGIN, SPELL_ORIGIN, WIZARD_2_POSITION, WIZARD_COOP_POSITION, WIZARD_POSITION,
    calculate_mp_defender_grid_position, calculate_mp_guest_defender_grid_position,
};

pub use tuning::{
    ALIGNMENT_STRENGTH, ATTACK_CYCLE_DURATION, ATTACK_DAMAGE, ATTACK_RANGE_MULTIPLIER,
    ATTACKER_HITBOX_HEIGHT, COHESION_STRENGTH, COLLISION_ITERATIONS,
    COOP_ATTACKER_EFFECTIVENESS_BONUS, DEFENDER_ACTIVATION_RANGE, DEFENDER_HITBOX_HEIGHT,
    EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT, EFFECTIVENESS_MAX,
    EFFECTIVENESS_MIN, GLOBAL_SPEED_MULTIPLIER, INITIAL_DEFENDER_COUNT, KINGS_GUARD_COUNT,
    KINGS_GUARD_ORBIT_RADIUS, MAX_OVERLAP_PERCENT, MELEE_SLOWDOWN_DISTANCE, MELEE_SLOWDOWN_FACTOR,
    MIN_DISTANCE_THRESHOLD, NEIGHBOR_DISTANCE, RETREAT_DURATION_SECS,
    RETREAT_TRIGGER_DISTANCE_PERCENT, SEPARATION_DISTANCE, SEPARATION_STRENGTH, STEERING_FORCE,
    UNIT_HEALTH, UNIT_MOVEMENT_SPEED, VELOCITY_DAMPING,
};

pub use spawn_math::{
    ARCHER_SPAWN_DEPTH_OFFSET, ASSASSIN_SPAWN_DEPTH_OFFSET, ATTACKER_SPAWN_POINTS,
    CENTER_STAGING_INDEX, DEFENDER_GRID_CENTER_ANGLE, DEFENDER_GRID_COLS,
    DEFENDER_GRID_GROUND_RANGE, DEFENDER_GRID_ROWS, GRID_ROW_DEPTH, STAGING_ACTIVATION_RADIUS,
    STAGING_POINT_COUNT, STAGING_POINTS, STAGING_SATISFACTION_RADIUS, STAGING_SPEEDUP,
    WAVE_ACTIVATION_THRESHOLD, WAVE_STAGING_TIMEOUT, attacker_spawn_position,
    calculate_defender_grid_position, defender_spawn_center,
};

pub use wave_tiers::{
    BOSS_CYCLE_LENGTH, WAVE_INTERVAL_SECONDS, boss_name_for_level, calculate_total_aerialists,
    calculate_total_archers, calculate_total_assassins, calculate_total_infantry,
    calculate_wave_count, cells_needed, distribute_units_to_cells, endless_effectiveness_bonus,
    endless_extra_archers, endless_extra_infantry, get_tier, get_tier_level, is_boss_level,
    is_lich_level,
};
