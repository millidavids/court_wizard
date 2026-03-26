use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_ORANGE, TINT_RED, UNIT_SCALE, tint};

// Visual appearance
pub const OGRE_COLOR: Color = tint(ATTACKER_BASE, TINT_ORANGE, 0.3);
pub const OGRE_ENRAGE_1_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.3);
pub const OGRE_ENRAGE_2_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.5);
pub const OGRE_ENRAGE_3_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.8);
pub const OGRE_ELLIPSE_WIDTH: f32 = 40.0 * UNIT_SCALE;
pub const OGRE_ELLIPSE_DEPTH: f32 = 60.0 * UNIT_SCALE;
pub const OGRE_RADIUS: f32 = 40.0 * UNIT_SCALE;
pub const OGRE_HITBOX_HEIGHT: f32 = 60.0 * UNIT_SCALE;

// Movement
pub const OGRE_MOVEMENT_SPEED: f32 = 160.0;

// Combat
pub const OGRE_HEALTH: f32 = 12000.0;
pub const OGRE_DAMAGE_MULTIPLIER: f32 = -0.5;
pub const OGRE_ATTACK_DAMAGE: f32 = 30.0;
pub const OGRE_ATTACK_COOLDOWN: f32 = 1.0;
pub const OGRE_MELEE_KNOCKBACK_SPEED: f32 = 800.0;
pub const OGRE_MELEE_KNOCKBACK_DURATION: f32 = 1.5;

// Enrage thresholds (HP ratio)
pub const ENRAGE_PHASE_1_THRESHOLD: f32 = 1.0 - 0.25;
pub const ENRAGE_PHASE_2_THRESHOLD: f32 = ENRAGE_PHASE_1_THRESHOLD - 0.25;
pub const ENRAGE_PHASE_3_THRESHOLD: f32 = ENRAGE_PHASE_2_THRESHOLD - 0.25;

// Enrage phase 1 bonuses
pub const ENRAGE_1_SPEED_BONUS: f32 = 0.15;
pub const ENRAGE_1_DAMAGE_BONUS: f32 = 0.25;

// Enrage phase 2 bonuses
pub const ENRAGE_2_SPEED_BONUS: f32 = ENRAGE_1_SPEED_BONUS * 2.0;
pub const ENRAGE_2_DAMAGE_BONUS: f32 = ENRAGE_1_DAMAGE_BONUS * 2.0;

// Enrage phase 3 bonuses
pub const ENRAGE_3_SPEED_BONUS: f32 = ENRAGE_2_SPEED_BONUS * 2.0;
pub const ENRAGE_3_DAMAGE_BONUS: f32 = ENRAGE_2_DAMAGE_BONUS * 2.0;

// Charge ability
pub const OGRE_CHARGE_COOLDOWN: f32 = 6.0;
pub const OGRE_CHARGE_TARGET_MIN_DISTANCE: f32 = 500.0;
pub const OGRE_CHARGE_TARGET_MAX_DISTANCE: f32 = OGRE_CHARGE_TARGET_MIN_DISTANCE * 2.0;
pub const OGRE_CHARGE_TELEGRAPH_DURATION: f32 = 3.0;
pub const OGRE_CHARGE_SPEED: f32 = 1200.0;
pub const OGRE_CHARGE_MAX_DISTANCE: f32 = OGRE_CHARGE_TARGET_MAX_DISTANCE * 0.8;
pub const OGRE_CHARGE_DAMAGE: f32 = 80.0;
pub const OGRE_CHARGE_KNOCKBACK_SPEED: f32 = 1000.0;
pub const OGRE_CHARGE_KNOCKBACK_DURATION: f32 = 1.0;
/// Hit detection accounts for the ogre's own radius; targets within
/// ogre radius + their hitbox radius are hit during the charge.
pub const OGRE_CHARGE_HIT_EXTRA: f32 = 20.0;
pub const OGRE_CHARGE_RECOVERY_DURATION: f32 = 0.5;
pub const OGRE_CHARGE_LANE_WIDTH: f32 = OGRE_ELLIPSE_WIDTH;
pub const OGRE_CHARGE_LINE_THICKNESS: f32 = 4.0;
pub const OGRE_CHARGE_INDICATOR_Y: f32 = 2.0;
pub const OGRE_CHARGE_LINE_COLOR: Color = Color::srgba(0.9, 0.1, 0.1, 0.8);
pub const OGRE_CHARGE_FILL_COLOR: Color = Color::srgba(0.9, 0.1, 0.1, 0.4);
pub const OGRE_CHARGE_PULSE_FREQUENCY: f32 = 2.0;
pub const OGRE_CHARGE_PULSE_AMPLITUDE: f32 = 0.3;

/// Fraction of melee damage the ogre actually takes (0.3 = 70% reduction).
pub const OGRE_MELEE_DAMAGE_REDUCTION: f32 = 0.3;
