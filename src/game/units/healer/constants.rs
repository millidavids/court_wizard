use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_GREEN, UNIT_SCALE, get_tier, get_tier_level, tint};

// ===== Visual =====
pub const ATTACKER_HEALER_COLOR: Color = tint(ATTACKER_BASE, TINT_GREEN, 0.6);
pub const HEALER_RADIUS: f32 = 8.0 * UNIT_SCALE;

// ===== Movement =====
pub const HEALER_MOVEMENT_SPEED: f32 = 110.0;

// ===== Health =====
pub const HEALER_HEALTH: f32 = 35.0;

// ===== Heal Bolt =====
pub const HEAL_BOLT_SPEED: f32 = 300.0;
pub const HEAL_BOLT_RADIUS: f32 = 5.0;
pub const HEAL_BOLT_LIFETIME: f32 = 5.0;
pub const HEAL_BOLT_COLOR: Color = Color::srgb(0.2, 0.9, 0.3);

// ===== Healing =====
pub const HEAL_RANGE: f32 = 400.0;
pub const HEAL_COOLDOWN: f32 = 4.0;

// ===== Spawn =====

/// Tier at which attacker healers start appearing.
pub const HEALER_START_TIER: u32 = 3;

/// Calculates the number of attacker healers for a given level.
/// Returns 0 below tier 3, then scales with tier_level (1 at tier_level 1, +1 per tier_level).
pub const fn calculate_attacker_healers(level: u32) -> u32 {
    let tier = get_tier(level);
    if tier < HEALER_START_TIER {
        0
    } else {
        get_tier_level(level)
    }
}
