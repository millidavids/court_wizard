use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_ORANGE, TINT_PURPLE, UNIT_SCALE, tint};

// ===== Visual Appearance =====

/// Fire-orange for Justina.
pub const JUSTINA_COLOR: Color = tint(ATTACKER_BASE, TINT_ORANGE, 0.5);
/// Mind-purple for Martina.
pub const MARTINA_COLOR: Color = tint(ATTACKER_BASE, TINT_PURPLE, 0.5);
/// Physical-brown for Josephina.
pub const JOSEPHINA_COLOR: Color = tint(ATTACKER_BASE, Color::srgb(0.6, 0.35, 0.15), 0.5);

pub const HAG_ELLIPSE_WIDTH: f32 = 25.0 * UNIT_SCALE;
pub const HAG_ELLIPSE_DEPTH: f32 = 35.0 * UNIT_SCALE;
pub const HAG_RADIUS: f32 = 25.0 * UNIT_SCALE;
pub const HAG_HITBOX_HEIGHT: f32 = 35.0 * UNIT_SCALE;

// ===== Movement =====

pub const HAG_MOVEMENT_SPEED: f32 = 100.0;
/// Minimum distance hags try to maintain from each other (world units).
pub const HAG_SEPARATION_DISTANCE: f32 = 300.0 * UNIT_SCALE;
/// Strength of the inter-hag separation force.
pub const HAG_SEPARATION_STRENGTH: f32 = 150.0;

// ===== Combat =====

pub const HAG_HEALTH: f32 = 6000.0;
pub const HAG_DAMAGE_MULTIPLIER: f32 = -0.3;
pub const HAG_ATTACK_DAMAGE: f32 = 20.0;
pub const HAG_ATTACK_COOLDOWN: f32 = 1.2;

// ===== Eye System =====

/// Base interval between eye transfers (seconds).
pub const EYE_TRANSFER_BASE_INTERVAL: f32 = 9.0;
/// Random variance on eye transfer interval (+/- this many seconds).
pub const EYE_TRANSFER_VARIANCE: f32 = 1.0;

/// Y offset for eye visuals above the hag sprite.
pub const EYE_VISUAL_OFFSET_Y: f32 = 40.0 * UNIT_SCALE;
/// Radius of the eye visual sphere.
pub const EYE_VISUAL_RADIUS: f32 = 16.0 * UNIT_SCALE;
/// Spacing between the two eyes when a hag has both.
pub const EYE_VISUAL_SPACING: f32 = 12.0 * UNIT_SCALE;

/// Gold color for the invulnerability eye.
pub const INVULNERABILITY_EYE_COLOR: Color = Color::srgb(1.0, 0.85, 0.0);
/// Bright cyan-blue color for the ability eye.
pub const ABILITY_EYE_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);

/// Duration for an eye to arc between hags (seconds).
pub const EYE_TOSS_FLIGHT_DURATION: f32 = 0.8;
/// Peak height of the parabolic arc for eye toss (world units).
pub const EYE_TOSS_ARC_HEIGHT: f32 = 80.0;
/// Duration for blind hag random wandering direction change (seconds).
pub const BLIND_WANDER_DIRECTION_INTERVAL: f32 = 2.0;


// ===== Death & Resurrection =====

/// Percentage of max HP healed on resurrection.
pub const RESURRECT_HEAL_PERCENT: f32 = 0.15;
/// Speed bonus for the last surviving hag (enraged).
pub const ENRAGE_SPEED_BONUS: f32 = 0.35;

// ===== Justina Abilities =====

/// Chain lightning cooldown (seconds).
pub const CHAIN_LIGHTNING_COOLDOWN: f32 = 1.0;
/// Chain lightning initial target range.
pub const CHAIN_LIGHTNING_RANGE: f32 = 250.0;
/// Chain lightning damage per hit.
pub const CHAIN_LIGHTNING_DAMAGE: f32 = 15.0;

/// Fireball cooldown (seconds).
pub const FIREBALL_COOLDOWN: f32 = 2.0;
/// Number of fireballs per cast.
pub const FIREBALL_COUNT: u32 = 2;
/// Fireball projectile speed.
pub const FIREBALL_SPEED: f32 = 800.0;
/// Fireball explosion damage per tick.
pub const FIREBALL_DAMAGE: f32 = 20.0;
/// Fireball explosion radius.
pub const FIREBALL_EXPLOSION_RADIUS: f32 = 40.0;
/// Fireball projectile collision radius.
pub const FIREBALL_COLLISION_RADIUS: f32 = 15.0;
/// Fireball visual mesh radius.
pub const FIREBALL_VISUAL_RADIUS: f32 = 8.0;

// ===== Josephina Abilities =====

/// Leap cooldown (seconds).
pub const LEAP_COOLDOWN: f32 = 2.5;
/// Maximum distance Josephina can leap (world units).
pub const LEAP_MAX_RANGE: f32 = 250.0;
/// Leap flight duration (seconds).
pub const LEAP_FLIGHT_DURATION: f32 = 0.6;
/// Leap maximum height (world units).
pub const LEAP_MAX_HEIGHT: f32 = 200.0;
/// Leap knockback radius on landing.
pub const LEAP_KNOCKBACK_RADIUS: f32 = 80.0;
/// Leap knockback speed.
pub const LEAP_KNOCKBACK_SPEED: f32 = 600.0;
/// Leap knockback decay duration.
pub const LEAP_KNOCKBACK_DURATION: f32 = 1.0;

/// Vicious mauling duration (seconds).
pub const MAULING_DURATION: f32 = 1.0;
/// Corpse consume duration (seconds).
pub const CORPSE_CONSUME_DURATION: f32 = 3.0;
/// Corpse consume heal amount (fraction of max HP).
pub const CORPSE_CONSUME_HEAL_PERCENT: f32 = 0.10;

// ===== Martina Abilities =====

/// Teleport pull cooldown (seconds).
pub const TELEPORT_PULL_COOLDOWN: f32 = 2.0;
/// Number of defenders teleported per pull.
pub const TELEPORT_PULL_COUNT: u32 = 5;

/// Radius of Martina's mind control aura (world units).
pub const MIND_CONTROL_AURA_RADIUS: f32 = 100.0;
/// Maximum number of mind-controlled units at once.
pub const MIND_CONTROL_MAX_CONTROLLED: u32 = 20;
/// Martina's aura color (translucent purple).
pub const MIND_CONTROL_AURA_COLOR: Color = Color::srgba(0.7, 0.2, 1.0, 0.15);
/// Damage dealt by mind-controlled units per attack.
pub const MIND_CONTROL_COMBAT_DAMAGE: f32 = 5.0;
/// Range at which Josephina will seek a corpse to consume.
pub const CORPSE_CONSUME_RANGE: f32 = 60.0;
/// Health threshold (fraction of max) below which Josephina will consume corpses.
pub const CORPSE_CONSUME_HEALTH_THRESHOLD: f32 = 0.9;

// ===== Spawn Positions =====

/// Grid columns for the 3 hags (row 0).
pub const JUSTINA_COL: u32 = 1;
pub const MARTINA_COL: u32 = 3;
pub const JOSEPHINA_COL: u32 = 5;
