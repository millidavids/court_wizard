use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// ===== Visual Appearance =====

/// Drives the 4× sprite quad. Same naming convention as the lich/hags.
pub const DARK_MAGE_ELLIPSE_WIDTH: f32 = 30.0 * UNIT_SCALE;
pub const DARK_MAGE_RADIUS: f32 = 30.0 * UNIT_SCALE;
pub const DARK_MAGE_HITBOX_HEIGHT: f32 = 50.0 * UNIT_SCALE;

// ===== Sprite Sheet Layout =====
// Both the floating and casting sheets are 4 columns × 2 rows (512×256 px).
// Row 0 = facing the camera; row 1 = facing away.

pub(super) const DARK_MAGE_SHEET_COLUMNS: usize = 4;
pub(super) const DARK_MAGE_SHEET_ROWS: usize = 2;
pub(super) const DARK_MAGE_FRAME_UV: Vec2 = Vec2::new(
    1.0 / DARK_MAGE_SHEET_COLUMNS as f32,
    1.0 / DARK_MAGE_SHEET_ROWS as f32,
);

/// Maps `FacingDirection` `[Forward, Back, Left, Right]` to sprite-sheet rows.
/// Lateral movement collapses to row 0 so the dark mage keeps facing the camera;
/// only the rear-arc Back direction uses row 1.
pub(super) const DARK_MAGE_DIRECTION_ROWS: [usize; 4] = [0, 1, 0, 0];

/// Sprite quad — frames are 128×128 (1:1), so the quad is square.
pub(super) const DARK_MAGE_SPRITE_WIDTH: f32 = DARK_MAGE_ELLIPSE_WIDTH * 4.0;
pub(super) const DARK_MAGE_SPRITE_HEIGHT: f32 = DARK_MAGE_ELLIPSE_WIDTH * 4.0;

// ===== Hover Bob =====

/// Base offset above the ground (added to the spawn-time Y).
pub(super) const DARK_MAGE_FLOAT_BASE_OFFSET: f32 = 30.0 * UNIT_SCALE;
/// Sinusoidal bob amplitude.
pub(super) const DARK_MAGE_FLOAT_AMPLITUDE: f32 = 4.0 * UNIT_SCALE;
/// Sinusoidal bob frequency in Hz.
pub(super) const DARK_MAGE_FLOAT_FREQUENCY_HZ: f32 = 1.2;

// ===== Facing =====

/// `cos(60°) = 0.5` — show the back-of-mage row only when moving in a 120° arc
/// directly away from the camera.
pub(super) const DARK_MAGE_BACK_FACING_THRESHOLD: f32 = 0.5;

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
/// Radius of the aura-bubble VFX spawned at the source and destination.
/// Sized to wrap around the dark mage sprite.
pub(super) const TELEPORT_BUBBLE_RADIUS: f32 = 200.0;
/// NDC bounds margin for camera-visibility test on teleport destinations
/// (-1 = left/bottom edge, +1 = right/top edge). 0.85 leaves a small safety
/// inset so the dark mage doesn't end up flush with the screen edge.
pub(super) const TELEPORT_NDC_MARGIN: f32 = 0.85;

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
/// Time the explosion sphere takes to grow from a point to its full radius.
pub const METEOR_EXPLOSION_GROWTH_TIME: f32 = 0.15;
/// Number of procedural fire-orange smoke billboards spawned at the impact site.
pub const METEOR_FIRE_PARTICLE_COUNT: usize = 18;
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
/// Number of lightning bolt visuals spawned along the corridor.
pub const LIGHTNING_BOLT_COUNT: usize = 3;

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

// ===== Spell Effect Colors =====

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
