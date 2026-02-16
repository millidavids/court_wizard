//! Constants for the elite and commander upgrade system.
//!
//! This module contains all configuration values for randomly upgrading
//! attacker units (infantry and archers) into elites and commanders.

use bevy::prelude::*;

// ============================================================================
// Elite Upgrade Probability
// ============================================================================

/// Minimum level at which elites can spawn.
pub const ELITE_START_LEVEL: u32 = 3;

/// Base probability of elite upgrade at the start level (10%).
pub const ELITE_BASE_CHANCE: f32 = 0.10;

/// Probability increase per level (+5% per level).
pub const ELITE_CHANCE_PER_LEVEL: f32 = 0.05;

/// Maximum probability for elite upgrades (60% cap).
/// Prevents entire army from being elite at high levels.
pub const ELITE_MAX_CHANCE: f32 = 0.60;

// ============================================================================
// Elite Caps
// ============================================================================

/// Maximum number of elite infantry units that can spawn.
/// Set to ~70% of base infantry count at level 1 (60 * 0.7 ≈ 42, rounded to 60 for safety).
pub const MAX_ELITE_INFANTRY: u32 = 60;

/// Maximum number of elite archer units that can spawn.
/// Set to ~75% of archer count at level 10 (30 * 0.75 ≈ 23, rounded to 30 for safety).
pub const MAX_ELITE_ARCHERS: u32 = 30;

// ============================================================================
// Commander Upgrade Probability
// ============================================================================

/// Minimum level at which commanders can spawn.
pub const COMMANDER_START_LEVEL: u32 = 5;

/// Base probability of commander upgrade at the start level (3%).
pub const COMMANDER_BASE_CHANCE: f32 = 0.03;

/// Probability increase per level (+2% per level).
pub const COMMANDER_CHANCE_PER_LEVEL: f32 = 0.02;

/// Maximum probability for commander upgrades (20% cap).
/// Commanders are rare, powerful units - kept at lower cap than elites.
pub const COMMANDER_MAX_CHANCE: f32 = 0.20;

// ============================================================================
// Commander Caps
// ============================================================================

/// Maximum number of infantry commanders that can spawn.
/// Commanders are rare and powerful - hard cap to prevent overwhelming auras.
pub const MAX_COMMANDER_INFANTRY: u32 = 5;

/// Maximum number of archer commanders that can spawn.
/// Lower than infantry since archers are typically fewer in number.
pub const MAX_COMMANDER_ARCHERS: u32 = 3;

// ============================================================================
// Commander Aura Configuration (Attackers)
// ============================================================================

/// Radius of attacker commander auras.
/// Slightly smaller than King's 200-unit radius for balance.
pub const ATTACKER_COMMANDER_AURA_RADIUS: f32 = 150.0;

/// Damage buff percentage for units within attacker commander aura.
/// +30% damage (vs King's +50% for defenders).
pub const ATTACKER_COMMANDER_DAMAGE_BUFF: f32 = 0.30;

/// Speed buff percentage for units within attacker commander aura.
/// +15% speed (vs King's +25% for defenders).
pub const ATTACKER_COMMANDER_SPEED_BUFF: f32 = 0.15;

/// Color of the attacker commander aura ring (red, semi-transparent).
pub const ATTACKER_COMMANDER_AURA_COLOR: Color = Color::srgba(1.0, 0.3, 0.0, 0.08);

// ============================================================================
// Visual Differentiation - Elite Units
// ============================================================================

/// Color for elite infantry units (bright red).
pub const ELITE_INFANTRY_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);

/// Color for elite archer units (hot pink-red).
pub const ELITE_ARCHER_COLOR: Color = Color::srgb(1.0, 0.3, 0.5);

/// Size multiplier for elite units (30% larger than normal).
pub const ELITE_SIZE_MULTIPLIER: f32 = 1.3;

// ============================================================================
// Visual Differentiation - Commander Units
// ============================================================================

/// Color for commander infantry units (orange-gold).
pub const COMMANDER_INFANTRY_COLOR: Color = Color::srgb(1.0, 0.7, 0.0);

/// Color for commander archer units (yellow-gold).
pub const COMMANDER_ARCHER_COLOR: Color = Color::srgb(1.0, 0.85, 0.0);

/// Size multiplier for commander units (60% larger than normal).
/// Commanders are the largest units to make them visually distinct.
pub const COMMANDER_SIZE_MULTIPLIER: f32 = 1.6;

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates the probability of elite upgrades for a given level.
///
/// Returns 0.0 for levels below ELITE_START_LEVEL.
/// Uses linear scaling with a maximum cap.
///
/// Formula: min(BASE + (level - START) * PER_LEVEL, MAX)
pub fn calculate_elite_chance(level: u32) -> f32 {
    if level < ELITE_START_LEVEL {
        return 0.0;
    }
    let chance = ELITE_BASE_CHANCE + (level - ELITE_START_LEVEL) as f32 * ELITE_CHANCE_PER_LEVEL;
    chance.min(ELITE_MAX_CHANCE)
}

/// Calculates the probability of commander upgrades for a given level.
///
/// Returns 0.0 for levels below COMMANDER_START_LEVEL.
/// Uses linear scaling with a maximum cap.
///
/// Formula: min(BASE + (level - START) * PER_LEVEL, MAX)
pub fn calculate_commander_chance(level: u32) -> f32 {
    if level < COMMANDER_START_LEVEL {
        return 0.0;
    }
    let chance =
        COMMANDER_BASE_CHANCE + (level - COMMANDER_START_LEVEL) as f32 * COMMANDER_CHANCE_PER_LEVEL;
    chance.min(COMMANDER_MAX_CHANCE)
}
