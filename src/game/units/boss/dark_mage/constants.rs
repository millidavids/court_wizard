use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_PURPLE, UNIT_SCALE, tint};

// ===== Visual Appearance =====

/// Base dark mage color (dark purple tint).
pub const DARK_MAGE_COLOR: Color = tint(ATTACKER_BASE, TINT_PURPLE, 0.5);
/// Enrage phase 1 color (deeper purple).
pub const DARK_MAGE_ENRAGE_1_COLOR: Color = tint(ATTACKER_BASE, TINT_PURPLE, 0.65);
/// Enrage phase 2 color (intense purple).
pub const DARK_MAGE_ENRAGE_2_COLOR: Color = tint(ATTACKER_BASE, TINT_PURPLE, 0.8);
/// Enrage phase 3 color (near-pure purple).
pub const DARK_MAGE_ENRAGE_3_COLOR: Color = tint(ATTACKER_BASE, TINT_PURPLE, 0.95);

pub const DARK_MAGE_ELLIPSE_WIDTH: f32 = 30.0 * UNIT_SCALE;
pub const DARK_MAGE_ELLIPSE_DEPTH: f32 = 45.0 * UNIT_SCALE;
pub const DARK_MAGE_RADIUS: f32 = 30.0 * UNIT_SCALE;
pub const DARK_MAGE_HITBOX_HEIGHT: f32 = 50.0 * UNIT_SCALE;

// ===== Health & Combat =====

pub const DARK_MAGE_HEALTH: f32 = 8000.0;
/// Negative damage multiplier = takes less melee damage (like ogre).
pub const DARK_MAGE_DAMAGE_MULTIPLIER: f32 = -0.3;
/// Fraction of melee damage the Dark Mage actually takes (0.4 = 60% reduction).
pub const DARK_MAGE_MELEE_DAMAGE_REDUCTION: f32 = 0.4;

// ===== Teleport =====

/// Seconds between teleports.
pub const TELEPORT_COOLDOWN: f32 = 7.0;
/// Minimum distance to teleport from current position.
pub const TELEPORT_MIN_DISTANCE: f32 = 400.0;
/// Minimum distance from castle position when choosing teleport destination.
pub const TELEPORT_MIN_CASTLE_DISTANCE: f32 = 500.0;

// ===== Enrage Thresholds (HP ratio) =====

pub const ENRAGE_PHASE_1_THRESHOLD: f32 = 0.75;
pub const ENRAGE_PHASE_2_THRESHOLD: f32 = 0.50;
pub const ENRAGE_PHASE_3_THRESHOLD: f32 = 0.25;

/// Cooldown reduction multiplier per enrage phase (applied to all spell cooldowns).
pub const ENRAGE_1_COOLDOWN_MULT: f32 = 0.85;
pub const ENRAGE_2_COOLDOWN_MULT: f32 = 0.70;
pub const ENRAGE_3_COOLDOWN_MULT: f32 = 0.55;

// ===== Dark Meteor =====

/// Cooldown between meteor casts.
pub const METEOR_COOLDOWN: f32 = 14.0;
/// Telegraph duration (red circle grows brighter).
pub const METEOR_TELEGRAPH_DURATION: f32 = 4.5;
/// Radius of the meteor explosion.
pub const METEOR_RADIUS: f32 = 200.0;
/// One-shot explosion damage.
pub const METEOR_DAMAGE: f32 = 60.0;
/// Duration of the explosion visual.
pub const METEOR_EXPLOSION_DURATION: f32 = 0.8;
/// Height from which the meteor projectile falls (above camera, off-screen).
pub const METEOR_FALL_HEIGHT: f32 = 2000.0;
/// Size of the falling meteor projectile visual.
pub const METEOR_PROJECTILE_RADIUS: f32 = 30.0;

// ===== Shadow Lightning =====

/// Cooldown between lightning casts.
pub const LIGHTNING_COOLDOWN: f32 = 10.0;
/// Telegraph duration (red corridor pulses).
pub const LIGHTNING_TELEGRAPH_DURATION: f32 = 3.5;
/// Width of the lightning corridor.
pub const LIGHTNING_CORRIDOR_WIDTH: f32 = 80.0;
/// Length of the lightning corridor.
pub const LIGHTNING_CORRIDOR_LENGTH: f32 = 600.0;
/// Damage dealt to units in the corridor.
pub const LIGHTNING_DAMAGE: f32 = 40.0;
/// Duration of the lightning strike visual.
pub const LIGHTNING_STRIKE_DURATION: f32 = 0.5;
/// Height of the lightning bolt visual (tall enough to originate off-camera).
pub const LIGHTNING_BOLT_HEIGHT: f32 = 2000.0;

// ===== Plague Cloud =====

/// Cooldown between plague cloud casts.
pub const PLAGUE_COOLDOWN: f32 = 18.0;
/// Telegraph duration (red circle expands).
pub const PLAGUE_TELEGRAPH_DURATION: f32 = 4.0;
/// Radius of the persistent plague cloud.
pub const PLAGUE_RADIUS: f32 = 250.0;
/// Damage per tick while standing in the cloud.
pub const PLAGUE_DAMAGE_PER_TICK: f32 = 8.0;
/// Time between damage ticks.
pub const PLAGUE_TICK_INTERVAL: f32 = 0.5;
/// How long the plague cloud persists after being placed.
pub const PLAGUE_DURATION: f32 = 8.0;
/// Flow field cost for pathfinding avoidance of the persistent cloud.
pub const PLAGUE_HAZARD_COST: f32 = 15.0;

// ===== Telegraph Indicator Visuals =====

/// Y position of telegraph indicators (just above ground).
pub const INDICATOR_Y: f32 = 2.0;
/// Base color of telegraph indicators (semi-transparent red).
pub const INDICATOR_BASE_COLOR: Color = Color::srgba(0.8, 0.1, 0.05, 0.15);
/// Emissive pulse frequency in Hz.
pub const INDICATOR_PULSE_FREQUENCY: f32 = 2.5;
/// Peak emissive intensity at full telegraph.
pub const INDICATOR_EMISSIVE_MAX: f32 = 8.0;

// ===== Spell Effect Colors =====

/// Meteor explosion fill color.
pub const METEOR_FILL_COLOR: Color = Color::srgba(0.9, 0.3, 0.05, 0.3);
/// Lightning strike color.
pub const LIGHTNING_FILL_COLOR: Color = Color::srgba(0.6, 0.5, 1.0, 0.4);
/// Plague cloud persistent zone color.
pub const PLAGUE_ZONE_COLOR: Color = Color::srgba(0.2, 0.6, 0.1, 0.25);

// ===== Approach Phase =====

/// Movement speed while walking from tunnel to battlefield.
pub const DARK_MAGE_APPROACH_SPEED: f32 = 320.0;
/// X position at which the Dark Mage stops approaching and starts casting.
/// Should be well onto the battlefield where defenders are visible.
pub const DARK_MAGE_APPROACH_TARGET_X: f32 = 0.0;

// ===== Targeting =====

/// Minimum distance from self when choosing spell targets.
pub const MIN_TARGET_DISTANCE_FROM_SELF: f32 = 150.0;

// ===== Visible Area Bounds (for teleport and spell targeting) =====

/// Approximate visible area bounds on the XZ ground plane.
/// These keep the Dark Mage and his spells on-screen.
pub const VISIBLE_MIN_X: f32 = -2200.0;
pub const VISIBLE_MAX_X: f32 = 2200.0;
pub const VISIBLE_MIN_Z: f32 = -2200.0;
pub const VISIBLE_MAX_Z: f32 = 800.0;

/// Maximum spell targeting range from the Dark Mage's position.
pub const MAX_SPELL_RANGE: f32 = 800.0;
