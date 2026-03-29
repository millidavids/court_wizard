use bevy::prelude::*;

use crate::game::constants::{TINT_PURPLE, UNDEAD_BASE, UNIT_SCALE, tint};

// ===== Visual Appearance =====

/// Lich body color — dark necrotic purple.
pub const LICH_COLOR: Color = tint(UNDEAD_BASE, TINT_PURPLE, 0.5);

/// Lich Phase 2 color — brighter, more menacing.
pub const LICH_COMBAT_COLOR: Color = tint(UNDEAD_BASE, Color::srgb(0.3, 0.9, 0.3), 0.4);

/// Lich ellipse width (ground shadow).
pub const LICH_ELLIPSE_WIDTH: f32 = 35.0 * UNIT_SCALE;

/// Lich ellipse depth (ground shadow).
pub const LICH_ELLIPSE_DEPTH: f32 = 50.0 * UNIT_SCALE;

/// Collision/hitbox radius.
pub const LICH_RADIUS: f32 = 35.0 * UNIT_SCALE;

/// Hitbox height for targeting.
pub const LICH_HITBOX_HEIGHT: f32 = 55.0 * UNIT_SCALE;

// ===== Movement =====

/// Movement speed during approach and combat.
pub const LICH_MOVEMENT_SPEED: f32 = 160.0;

// ===== Health & Defense =====

/// Lich HP — very tanky in Phase 2.
pub const LICH_HEALTH: f32 = 15000.0;

/// Fraction of melee damage the Lich takes (0.3 = 70% reduction).
pub const LICH_MELEE_DAMAGE_REDUCTION: f32 = 0.3;

/// Damage multiplier (negative = takes less damage from non-spell sources).
pub const LICH_DAMAGE_MULTIPLIER: f32 = -0.5;

// ===== Soul Power =====

/// Maximum soul power needed to trigger Phase 2.
pub const SOUL_POWER_MAX: f32 = 100.0;

/// Soul power gained per undead kill.
pub const SOUL_POWER_PER_KILL: f32 = 1.0;

// ===== Summoning (Phase 1) =====

/// Seconds between summon waves.
pub const SUMMON_INTERVAL: f32 = 4.0;

/// Number of undead infantry per summon wave.
pub const SUMMON_WAVE_SIZE: u32 = 15;

/// Radius around the Lich at which summoned undead spawn.
pub const SUMMON_SPAWN_RADIUS: f32 = 200.0;

/// Health of summoned undead infantry.
pub const SUMMONED_UNDEAD_HEALTH: f32 = 80.0;

/// Movement speed of summoned undead.
pub const SUMMONED_UNDEAD_SPEED: f32 = 100.0;

// ===== Finger of Death Beam (Phase 2) =====

/// Seconds between beam attacks.
pub const BEAM_COOLDOWN: f32 = 3.0;

/// Damage dealt by the death beam.
pub const BEAM_DAMAGE: f32 = 500.0;

/// Beam width for hit detection.
pub const BEAM_WIDTH: f32 = 15.0;

/// Maximum beam length.
pub const BEAM_LENGTH: f32 = 5000.0;

/// King takes only 30% of FoD damage (70% resistance).
pub const KING_FOD_DAMAGE_MULTIPLIER: f32 = 0.3;

/// Fraction of defenders that must die before the Lich can target the King.
pub const KING_TARGET_THRESHOLD: f32 = 0.5;

// ===== Health Bar Colors =====

/// Soul power bar fill color (necrotic green).
pub const SOUL_POWER_BAR_COLOR: Color = Color::srgba(0.2, 0.8, 0.1, 0.8);

/// Soul power bar border color.
pub const SOUL_POWER_BAR_BORDER_COLOR: Color = Color::srgba(0.1, 0.5, 0.05, 1.0);
