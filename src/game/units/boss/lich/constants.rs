use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// ===== Visual Appearance =====

/// Canonical ellipse width — drives the 4× sprite quad and the spawn-Y formula.
pub const LICH_ELLIPSE_WIDTH: f32 = 35.0 * UNIT_SCALE;

/// Canonical ellipse depth — drives the 4× sprite quad and the spawn-Y formula.
pub const LICH_ELLIPSE_DEPTH: f32 = 50.0 * UNIT_SCALE;

/// Collision/hitbox radius — sized to encompass the lich's robed body width.
pub const LICH_RADIUS: f32 = 45.0 * UNIT_SCALE;

/// Hitbox height for targeting — matches ~90% of the 4× sprite quad height
/// so the cylinder visually wraps the rendered lich.
pub const LICH_HITBOX_HEIGHT: f32 = 180.0 * UNIT_SCALE;

// ===== Sprite Sheet Layout =====
// Both the floating and casting sheets are 4 columns × 2 rows.
// Row 0 = facing the camera; row 1 = facing away.

pub(super) const LICH_SHEET_COLUMNS: usize = 4;
pub(super) const LICH_SHEET_ROWS: usize = 2;
pub(super) const LICH_FRAME_UV: Vec2 = Vec2::new(
    1.0 / LICH_SHEET_COLUMNS as f32,
    1.0 / LICH_SHEET_ROWS as f32,
);

/// Maps `FacingDirection` `[Forward, Back, Left, Right]` to sprite-sheet rows.
/// Lateral movement collapses to row 0 so the lich keeps facing the camera;
/// only the rear-arc Back direction uses row 1 (lich's back to viewer).
pub(super) const LICH_DIRECTION_ROWS: [usize; 4] = [0, 1, 0, 0];

/// Sprite quad dimensions — match the hag pattern (ellipse-size × 4).
pub(super) const LICH_SPRITE_WIDTH: f32 = LICH_ELLIPSE_WIDTH * 4.0;
pub(super) const LICH_SPRITE_HEIGHT: f32 = LICH_ELLIPSE_DEPTH * 4.0;

// ===== Float / Hover =====

/// Base offset above the ground (added to the spawn-time Y).
pub(super) const LICH_FLOAT_BASE_OFFSET: f32 = 30.0 * UNIT_SCALE;
/// Sinusoidal bob amplitude.
pub(super) const LICH_FLOAT_AMPLITUDE: f32 = 4.0 * UNIT_SCALE;
/// Sinusoidal bob frequency in Hz.
pub(super) const LICH_FLOAT_FREQUENCY_HZ: f32 = 1.2;

// ===== Cast Wind-Up Timing =====

/// Pre-cast wind-up before a Raise Dead summon resolves.
pub(super) const LICH_RAISE_DEAD_CAST_DURATION: f32 = 3.0;
/// Pre-cast wind-up before a Finger of Death beam fires.
pub(super) const LICH_FINGER_OF_DEATH_CAST_DURATION: f32 = 0.6;

/// Threshold on `velocity.dot(camera_forward) / |velocity|` above which the
/// lich shows the back-of-lich row (sheet row 1). `cos(60°) = 0.5` ⇒ the
/// rear-facing row is used only when the lich is moving in a 120° arc
/// directly away from the camera. In Court Wizard's coordinate system,
/// `camera_forward` points into the screen, so `forward_dot > 0` means the
/// lich is moving away from the viewer — that's when the back is visible.
pub(super) const LICH_BACK_FACING_THRESHOLD: f32 = 0.5;

// ===== Movement =====

/// Movement speed during approach and combat.
pub const LICH_MOVEMENT_SPEED: f32 = 185.0;

// ===== Health & Defense =====

/// Lich HP — very tanky in Phase 2.
pub const LICH_HEALTH: f32 = 15000.0;

/// Fraction of melee damage the Lich takes (0.15 = 85% reduction).
pub const LICH_MELEE_DAMAGE_REDUCTION: f32 = 0.15;

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

/// Beam width for hit detection.
pub const BEAM_WIDTH: f32 = 15.0;

/// Maximum beam length.
pub const BEAM_LENGTH: f32 = 5000.0;

/// Maximum distance from the King at which the Lich is allowed to start
/// casting Finger of Death. The Lich must close this gap before firing.
pub const FOD_KING_RANGE: f32 = 800.0;

/// Fraction of defenders that must die before the King can take Finger of
/// Death damage. Below this threshold the King is fully immune to FoD even
/// if he is caught in a beam targeting another defender. Once it lifts, the
/// beam deals the King's full max health — a temp-HP shield (e.g. Guardian
/// Circle) is the counter-play.
pub const KING_FOD_IMMUNITY_THRESHOLD: f32 = 0.85;

// ===== Health Bar Colors =====

/// Soul power bar fill color (necrotic green).
pub const SOUL_POWER_BAR_COLOR: Color = Color::srgba(0.2, 0.8, 0.1, 0.8);

/// Soul power bar border color.
pub const SOUL_POWER_BAR_BORDER_COLOR: Color = Color::srgba(0.1, 0.5, 0.05, 1.0);
