use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_ORANGE, TINT_RED, tint};

// Visual appearance
pub const OGRE_COLOR: Color = tint(ATTACKER_BASE, TINT_ORANGE, 0.3);
pub const OGRE_ENRAGE_1_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.3);
pub const OGRE_ENRAGE_2_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.5);
pub const OGRE_ENRAGE_3_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.8);
pub const OGRE_ELLIPSE_WIDTH: f32 = 40.0;
pub const OGRE_ELLIPSE_DEPTH: f32 = 60.0;
pub const OGRE_RADIUS: f32 = 40.0;
pub const OGRE_HITBOX_HEIGHT: f32 = 60.0;

// Movement
pub const OGRE_MOVEMENT_SPEED: f32 = 110.0;

// Combat
pub const OGRE_HEALTH: f32 = 3000.0;
pub const OGRE_DAMAGE_MULTIPLIER: f32 = -0.5;
pub const OGRE_ATTACK_DAMAGE: f32 = 30.0;
pub const OGRE_ATTACK_COOLDOWN: f32 = 1.0;
pub const OGRE_MELEE_KNOCKBACK_SPEED: f32 = 800.0;
pub const OGRE_MELEE_KNOCKBACK_DURATION: f32 = 1.5;

// Enrage thresholds (HP ratio)
pub const ENRAGE_PHASE_1_THRESHOLD: f32 = 0.75;
pub const ENRAGE_PHASE_2_THRESHOLD: f32 = 0.50;
pub const ENRAGE_PHASE_3_THRESHOLD: f32 = 0.25;

// Enrage phase 1 bonuses
pub const ENRAGE_1_SPEED_BONUS: f32 = 0.15;
pub const ENRAGE_1_DAMAGE_BONUS: f32 = 0.25;

// Enrage phase 2 bonuses
pub const ENRAGE_2_SPEED_BONUS: f32 = 0.30;
pub const ENRAGE_2_DAMAGE_BONUS: f32 = 0.50;

// Enrage phase 3 bonuses
pub const ENRAGE_3_SPEED_BONUS: f32 = 0.50;
pub const ENRAGE_3_DAMAGE_BONUS: f32 = 1.00;
