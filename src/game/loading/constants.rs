//! Constants for the elite and commander upgrade system.
//!
//! This module contains all configuration values for randomly upgrading
//! attacker units (infantry and archers) into elites and commanders.

use crate::game::constants::{get_tier, get_tier_level};

// ============================================================================
// Elite Upgrade Probability
// ============================================================================

/// Minimum tier at which elites can spawn.
pub const ELITE_START_TIER: u32 = 1;

/// Base probability of elite upgrade at tier_level 1 of the start tier.
pub const ELITE_BASE_CHANCE: f32 = 0.00625;

/// Probability increase per tier_level (+1.25% per tier_level).
pub const ELITE_CHANCE_PER_TIER_LEVEL: f32 = 0.003125;

/// Maximum probability for elite upgrades (15% cap).
pub const ELITE_MAX_CHANCE: f32 = 0.15;

// ============================================================================
// Elite Caps
// ============================================================================

/// Maximum number of elite units (any type) that can spawn.
pub const MAX_ELITES: u32 = 90;

// ============================================================================
// Commander Upgrade Probability
// ============================================================================

/// Minimum tier at which commanders can spawn.
pub const COMMANDER_START_TIER: u32 = 2;

/// Base probability of commander upgrade at tier_level 1 of the start tier.
pub const COMMANDER_BASE_CHANCE: f32 = 0.00375;

/// Probability increase per tier_level (+1% per tier_level).
pub const COMMANDER_CHANCE_PER_TIER_LEVEL: f32 = 0.0025;

/// Maximum probability for commander upgrades (5% cap).
pub const COMMANDER_MAX_CHANCE: f32 = 0.05;

// ============================================================================
// Commander Caps
// ============================================================================

/// Maximum number of infantry commanders that can spawn.
pub const MAX_COMMANDER_INFANTRY: u32 = 5;

/// Maximum number of archer commanders that can spawn.
pub const MAX_COMMANDER_ARCHERS: u32 = 3;

// ============================================================================
// Commander Aura Configuration (Attackers)
// ============================================================================

/// Radius of attacker commander auras.
pub const ATTACKER_COMMANDER_AURA_RADIUS: f32 = 150.0;

/// Damage buff percentage for units within attacker commander aura.
pub const ATTACKER_COMMANDER_DAMAGE_BUFF: f32 = 0.30;

/// Speed buff percentage for units within attacker commander aura.
pub const ATTACKER_COMMANDER_SPEED_BUFF: f32 = 0.15;


// ============================================================================
// Visual Differentiation - Elite Units
// ============================================================================

/// Size multiplier for elite units (30% larger than normal).
pub const ELITE_SIZE_MULTIPLIER: f32 = 1.3;

// ============================================================================
// Visual Differentiation - Commander Units
// ============================================================================

/// Size multiplier for commander units (60% larger than normal).
pub const COMMANDER_SIZE_MULTIPLIER: f32 = 1.6;

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates the probability of elite upgrades for a given level.
///
/// Returns 0.0 for levels in tiers below ELITE_START_TIER.
/// Uses tier_level for linear scaling with a maximum cap.
pub fn calculate_elite_chance(level: u32) -> f32 {
    let tier = get_tier(level);
    if tier < ELITE_START_TIER {
        return 0.0;
    }
    let tier_level = get_tier_level(level);
    let chance = ELITE_BASE_CHANCE + (tier_level - 1) as f32 * ELITE_CHANCE_PER_TIER_LEVEL;
    chance.min(ELITE_MAX_CHANCE)
}

/// Calculates the probability of commander upgrades for a given level.
///
/// Returns 0.0 for levels in tiers below COMMANDER_START_TIER.
/// Uses tier_level for linear scaling with a maximum cap.
pub fn calculate_commander_chance(level: u32) -> f32 {
    let tier = get_tier(level);
    if tier < COMMANDER_START_TIER {
        return 0.0;
    }
    let tier_level = get_tier_level(level);
    let chance = COMMANDER_BASE_CHANCE + (tier_level - 1) as f32 * COMMANDER_CHANCE_PER_TIER_LEVEL;
    chance.min(COMMANDER_MAX_CHANCE)
}
