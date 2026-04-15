use crate::game::constants::{UNIT_SCALE, get_tier, get_tier_level};
use bevy::prelude::*;

// ===== Visual =====
pub const SHIELDER_RADIUS: f32 = 8.0 * UNIT_SCALE;

/// Sprite tint for shielders.
pub const SHIELDER_SPRITE_TINT: Color = Color::srgb(0.75, 0.65, 0.65);

// ===== Movement =====
pub const SHIELDER_MOVEMENT_SPEED: f32 = 125.0;

// ===== Health =====
pub const SHIELDER_HEALTH: f32 = 60.0;

// ===== Shielding =====
/// Range at which the shielder can apply a shield to an ally.
pub const SHIELD_RANGE: f32 = 200.0;
/// Cooldown between shield applications (seconds).
pub const SHIELD_COOLDOWN: f32 = 5.0;

/// Damage reduction multiplier for shielded units (20% reduction = 0.8x damage).
pub const SHIELDER_DAMAGE_REDUCTION: f32 = 0.8;

/// Duration of the shielder's pre-cast channel, in seconds.
pub(super) const SHIELDER_CAST_DURATION: f32 = 5.0;

/// Channel VFX — inward-imploding yellow particles.
pub(super) const SHIELDER_CHANNEL_PARTICLE_SPAWN_INTERVAL: f32 = 0.05;
pub(super) const SHIELDER_CHANNEL_PARTICLE_COUNT_PER_SPAWN: usize = 3;
pub(super) const SHIELDER_CHANNEL_PARTICLE_START_RADIUS: f32 = 18.0;
pub(super) const SHIELDER_CHANNEL_PARTICLE_MAX_RADIUS: f32 = 140.0;
pub(super) const SHIELDER_CHANNEL_PARTICLE_SIZE: f32 = 5.0;
pub(super) const SHIELDER_CHANNEL_PARTICLE_LIFETIME: f32 = 0.8;

// ===== Spawn =====

/// Tier at which attacker shielders start appearing.
pub const SHIELDER_START_TIER: u32 = 3;

/// Calculates the number of attacker shielders for a given level.
/// Returns 0 below tier 3, then scales with tier_level (1 at tier_level 1, +1 per tier_level).
pub const fn calculate_attacker_shielders(level: u32) -> u32 {
    let tier = get_tier(level);
    if tier < SHIELDER_START_TIER {
        0
    } else {
        get_tier_level(level)
    }
}
