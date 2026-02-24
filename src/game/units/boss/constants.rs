use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, TINT_ORANGE, TINT_RED, tint};

// Visual appearance
pub const BOSS_COLOR: Color = tint(ATTACKER_BASE, TINT_ORANGE, 0.3);
pub const BOSS_ENRAGE_1_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.3);
pub const BOSS_ENRAGE_2_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.5);
pub const BOSS_ENRAGE_3_COLOR: Color = tint(ATTACKER_BASE, TINT_RED, 0.8);
pub const BOSS_ELLIPSE_WIDTH: f32 = 40.0;
pub const BOSS_ELLIPSE_DEPTH: f32 = 60.0;
pub const BOSS_RADIUS: f32 = 40.0;
pub const BOSS_HITBOX_HEIGHT: f32 = 60.0; // Matches BOSS_ELLIPSE_DEPTH so hitbox aligns with visual sprite

// Movement
pub const BOSS_MOVEMENT_SPEED: f32 = 110.0;

// Combat
pub const BOSS_HEALTH: f32 = 3000.0;
pub const BOSS_DAMAGE_MULTIPLIER: f32 = -0.5; // Only used for enrage scaling base, not actual melee damage
pub const BOSS_ATTACK_DAMAGE: f32 = 30.0; // Flat damage per melee hit (ignores shared combat system)
pub const BOSS_ATTACK_COOLDOWN: f32 = 1.0; // Attacks every 1 second (twice as fast as normal 2s cycle)
pub const BOSS_MELEE_KNOCKBACK_SPEED: f32 = 800.0; // Initial knockback speed (units/s)
pub const BOSS_MELEE_KNOCKBACK_DURATION: f32 = 1.5; // How long the tumble lasts (seconds) — total ~600 units displacement

// Spawn interval
pub const BOSS_SPAWN_LEVEL_INTERVAL: u32 = 5;

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
