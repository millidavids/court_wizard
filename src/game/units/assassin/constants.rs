use crate::game::constants::UNIT_SCALE;
use bevy::prelude::*;

// ===== Movement =====

/// Assassin movement speed — 200% of infantry speed (200).
pub const ASSASSIN_MOVEMENT_SPEED: f32 = 400.0;

// ===== Combat =====

/// Damage multiplier when assassin is hit by archers (50% reduction).
pub const ARCHER_DAMAGE_REDUCTION: f32 = 0.5;

/// Damage multiplier when assassin is hit by infantry (20% increase).
pub const INFANTRY_DAMAGE_INCREASE: f32 = 1.2;

/// Damage multiplier when assassin attacks archers (instant kill).
pub const ASSASSIN_VS_ARCHER_DAMAGE: f32 = 1000.0;

/// Assassin attack speed bonus (2.0 = 3x attack speed).
pub const ASSASSIN_ATTACK_SPEED_BONUS: f32 = 2.0;

// ===== Health =====

/// Assassin health — same as standard units.
pub const ASSASSIN_HEALTH: f32 = 100.0;

// ===== Targeting =====

/// Distance at which direct targeting overtakes the flow field.
/// When an archer is closer than this, the assassin charges directly.
pub const TARGETING_CROSSOVER_DISTANCE: f32 = 150.0;

// ===== Flow Field =====

/// Cost multiplier applied to cells near infantry positions.
/// Higher values make assassins route further around infantry.
pub const INFANTRY_AVOIDANCE_COST: f32 = 30.0;

/// Avoidance radius (in grid cells) around friendly attacker infantry.
/// 10 cells = 100 world units — wide berth to flank around own infantry.
pub const ATTACKER_INFANTRY_AVOIDANCE_RADIUS: u32 = 10;

/// Avoidance radius (in grid cells) around enemy defender infantry.
/// 10 cells = 100 world units — wide berth to flank around defender formations.
pub const DEFENDER_INFANTRY_AVOIDANCE_RADIUS: u32 = 10;

// ===== Visual =====

/// Assassin sprite alpha (slightly translucent).
pub const ASSASSIN_ALPHA: f32 = 0.7;

/// Assassin sprite tint — darker, shadowy version of attacker tint.
pub const ASSASSIN_SPRITE_TINT: Color = Color::srgba(0.5, 0.4, 0.55, ASSASSIN_ALPHA);

pub const ASSASSIN_RADIUS: f32 = 8.0 * UNIT_SCALE;

