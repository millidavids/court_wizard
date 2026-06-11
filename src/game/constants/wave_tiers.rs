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
pub const BOSS_CYCLE_LENGTH: u32 = 5;

/// Returns the boss name for a given boss level, or None if not a boss level.
pub fn boss_name_for_level(level: u32) -> Option<&'static str> {
    if !is_boss_level(level) {
        return None;
    }
    Some(match get_tier(level) % BOSS_CYCLE_LENGTH {
        0 => "Ogre",
        1 => "The Lich",
        2 => "Dark Mage",
        3 => "Hags",
        4 => "Ray",
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
