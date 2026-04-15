use bevy::prelude::*;

use crate::game::constants::{UNIT_SCALE, get_tier, get_tier_level};

// ===== Visual =====
pub const HEALER_RADIUS: f32 = 8.0 * UNIT_SCALE;

/// Sprite tint for healers.
pub const HEALER_SPRITE_TINT: Color = Color::srgb(0.75, 0.65, 0.65);

// ===== Movement =====
pub const HEALER_MOVEMENT_SPEED: f32 = 125.0;

// ===== Health =====
pub const HEALER_HEALTH: f32 = 70.0;

// ===== Heal Bolt =====
pub const HEAL_BOLT_SPEED: f32 = 300.0;
pub const HEAL_BOLT_RADIUS: f32 = 5.0;
pub const HEAL_BOLT_LIFETIME: f32 = 5.0;
pub const HEAL_BOLT_COLOR: Color = Color::srgb(0.2, 0.9, 0.3);

// ===== Healing =====
pub const HEAL_RANGE: f32 = 400.0;
pub const HEAL_COOLDOWN: f32 = 4.0;

/// Duration of the pre-heal channel cast, in seconds.
pub(super) const HEALER_CAST_DURATION: f32 = 5.0;

/// Channel VFX — inward-imploding green particles.
pub(super) const HEALER_CHANNEL_PARTICLE_SPAWN_INTERVAL: f32 = 0.05;
pub(super) const HEALER_CHANNEL_PARTICLE_COUNT_PER_SPAWN: usize = 3;
pub(super) const HEALER_CHANNEL_PARTICLE_START_RADIUS: f32 = 18.0;
pub(super) const HEALER_CHANNEL_PARTICLE_MAX_RADIUS: f32 = 140.0;
pub(super) const HEALER_CHANNEL_PARTICLE_SIZE: f32 = 5.0;
pub(super) const HEALER_CHANNEL_PARTICLE_LIFETIME: f32 = 0.8;

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
