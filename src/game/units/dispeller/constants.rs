use bevy::prelude::*;

use crate::game::constants::{
    ATTACKER_BASE, TINT_BLUE, UNIT_SCALE, get_tier, get_tier_level, tint,
};

// ===== Visual =====
pub const ATTACKER_DISPELLER_COLOR: Color = tint(ATTACKER_BASE, TINT_BLUE, 0.6);
pub const DISPELLER_RADIUS: f32 = 8.0 * UNIT_SCALE;

// ===== Movement =====
pub const DISPELLER_MOVEMENT_SPEED: f32 = 120.0;

// ===== Health =====
pub const DISPELLER_HEALTH: f32 = 80.0;

// ===== Dispel =====
pub const DISPEL_RANGE: f32 = 150.0;
pub const DISPEL_COOLDOWN: f32 = 1.5;

// ===== Ranged Attack =====
pub const BOLT_DAMAGE: f32 = 5.0;
pub const BOLT_SPEED: f32 = 400.0;
pub const BOLT_RADIUS: f32 = 4.0;
pub const BOLT_LIFETIME: f32 = 3.0;
pub const BOLT_COLOR: Color = Color::srgb(0.3, 0.5, 1.0);
pub const ATTACK_RANGE: f32 = 500.0;
pub const ATTACK_COOLDOWN: f32 = 2.0;

// ===== Spawn =====

/// Tier at which attacker dispellers start appearing.
pub const ATTACKER_DISPELLER_START_TIER: u32 = 2;

/// Calculates the number of attacker dispellers for a given level.
/// Returns 0 below tier 3, then scales with tier_level (1 at tier_level 1, +1 per tier_level).
pub const fn calculate_attacker_dispellers(level: u32) -> u32 {
    let tier = get_tier(level);
    if tier < ATTACKER_DISPELLER_START_TIER {
        0
    } else {
        get_tier_level(level)
    }
}
