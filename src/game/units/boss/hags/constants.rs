use bevy::prelude::*;

use crate::game::constants::{UNIT_SCALE, tint};

// ===== Visual Appearance =====

/// Sprite overlay hues — matched to each hag's health bar color so the
/// player can immediately associate the overlay tint with the right bar.
/// Kept subtle so the underlying sprite art still reads.
const HAG_TINT_STRENGTH: f32 = 0.4;
const HAG_TINT_WHITE: Color = Color::srgb(1.0, 1.0, 1.0);

/// Justina's health-bar hue (orange).
const JUSTINA_BAR_HUE: Color = Color::srgb(0.9, 0.4, 0.1);
/// Martina's health-bar hue (purple).
const MARTINA_BAR_HUE: Color = Color::srgb(0.5, 0.15, 0.7);
/// Josephina's health-bar hue (green).
const JOSEPHINA_BAR_HUE: Color = Color::srgb(0.25, 0.75, 0.2);

pub const JUSTINA_COLOR: Color = tint(HAG_TINT_WHITE, JUSTINA_BAR_HUE, HAG_TINT_STRENGTH);
pub const MARTINA_COLOR: Color = tint(HAG_TINT_WHITE, MARTINA_BAR_HUE, HAG_TINT_STRENGTH);
pub const JOSEPHINA_COLOR: Color = tint(HAG_TINT_WHITE, JOSEPHINA_BAR_HUE, HAG_TINT_STRENGTH);

pub const HAG_ELLIPSE_WIDTH: f32 = 25.0 * UNIT_SCALE;
pub const HAG_ELLIPSE_DEPTH: f32 = 35.0 * UNIT_SCALE;
pub const HAG_RADIUS: f32 = 25.0 * UNIT_SCALE;
pub const HAG_HITBOX_HEIGHT: f32 = 35.0 * UNIT_SCALE;

// ===== Sprite Sheet =====

/// Hag walking sprite sheet dimensions: 4 frames × 4 directions, 128px frames.
pub(super) const HAG_SHEET_WIDTH: f32 = 512.0;
pub(super) const HAG_SHEET_HEIGHT: f32 = 512.0;
pub(super) const HAG_SHEET_FRAME_SIZE: f32 = 128.0;
pub(super) const HAG_SHEET_COLUMNS: usize = 4;
/// UV size of one frame on every hag sheet (walking, attacking, casting).
pub(super) const HAG_FRAME_UV: Vec2 = Vec2::new(
    HAG_SHEET_FRAME_SIZE / HAG_SHEET_WIDTH,
    HAG_SHEET_FRAME_SIZE / HAG_SHEET_HEIGHT,
);
/// Number of frames in the hag attack sprite sheet (per row).
pub(super) const HAG_ATTACK_COLUMNS: usize = 4;
/// Number of frames in the hag casting sprite sheet (per row).
pub(super) const HAG_CASTING_COLUMNS: usize = 4;
/// Walking/attack sheet rows: row 0 = facing the camera (gameplay-back),
/// row 1 = facing away (gameplay-forward), row 2 = facing left, row 3 = facing right.
/// Indexed by `FacingDirection as usize` order: [Forward, Back, Left, Right].
pub(super) const HAG_DIRECTION_ROWS: [usize; 4] = [1, 0, 2, 3];
/// Casting sheet rows: row 0 = facing the camera (gameplay-back), row 1 = right,
/// row 2 = left, row 3 = facing away (gameplay-forward).
/// Indexed by `FacingDirection as usize` order: [Forward, Back, Left, Right].
pub(super) const HAG_CASTING_DIRECTION_ROWS: [usize; 4] = [3, 0, 2, 1];

/// Visual sprite quad size for hags (sized to roughly match the old ellipse footprint).
pub(super) const HAG_SPRITE_WIDTH: f32 = HAG_ELLIPSE_WIDTH * 4.0;
pub(super) const HAG_SPRITE_HEIGHT: f32 = HAG_ELLIPSE_DEPTH * 4.0;

/// Visual sprite quad size for hag eyes (smaller than the collision radius so
/// the floating eye reads as a small icon above the hag rather than a large orb).
pub(super) const EYE_SPRITE_SIZE: f32 = EYE_VISUAL_RADIUS;

// ===== Movement =====

pub const HAG_MOVEMENT_SPEED: f32 = 230.0;
/// Minimum distance hags try to maintain from each other (world units).
pub const HAG_SEPARATION_DISTANCE: f32 = 400.0 * UNIT_SCALE;
/// Strength of the inter-hag separation force, in velocity units per second.
/// Falloff is quadratic — gentle at the outer edge, much stronger when sprites
/// are about to overlap.
pub const HAG_SEPARATION_STRENGTH: f32 = 320.0;

/// Distance from the king at which Justina stops advancing and kites with
/// her ranged abilities (chain lightning + fireball).
pub const JUSTINA_KITE_DISTANCE: f32 = 800.0;

// ===== Combat =====

pub const HAG_HEALTH: f32 = 12000.0;
pub const HAG_DAMAGE_MULTIPLIER: f32 = -0.6;
pub const HAG_ATTACK_DAMAGE: f32 = 20.0;
pub const HAG_ATTACK_COOLDOWN: f32 = 1.2;

// ===== Eye System =====

/// Base interval between eye transfers (seconds).
pub const EYE_TRANSFER_BASE_INTERVAL: f32 = 9.0;
/// Random variance on eye transfer interval (+/- this many seconds).
pub const EYE_TRANSFER_VARIANCE: f32 = 1.0;

/// Y offset for eye visuals above the hag sprite.
pub const EYE_VISUAL_OFFSET_Y: f32 = 60.0 * UNIT_SCALE;
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
pub const FIREBALL_EXPLOSION_RADIUS: f32 = 120.0;
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

/// Pause duration after landing before Josephina attacks (seconds).
pub const LEAP_LANDING_PAUSE: f32 = 0.3;
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
/// Martina must be within this distance of the king to cast teleport pull.
/// (Once the cast fires, any defender on the map is eligible to be pulled.)
pub const TELEPORT_PULL_KING_RANGE: f32 = 800.0;

/// Radius of Martina's mind control aura (world units).
pub const MIND_CONTROL_AURA_RADIUS: f32 = 140.0;
/// Maximum number of mind-controlled units at once.
pub const MIND_CONTROL_MAX_CONTROLLED: u32 = 20;
/// Martina's aura color (translucent purple).
pub const MIND_CONTROL_AURA_COLOR: Color = Color::srgba(0.7, 0.2, 1.0, 0.15);
/// Damage dealt by mind-controlled units per attack (re-export from mind_control spell).
pub use crate::game::units::wizard::spells::mind_control::constants::COMBAT_DAMAGE as MIND_CONTROL_COMBAT_DAMAGE;
/// Range at which Josephina will seek a corpse to consume.
pub const CORPSE_CONSUME_RANGE: f32 = 60.0;
/// Health threshold (fraction of max) below which Josephina will consume corpses.
pub const CORPSE_CONSUME_HEALTH_THRESHOLD: f32 = 0.9;

// ===== Spawn Positions =====

/// Grid columns for the 3 hags (row 0).
pub const JUSTINA_COL: u32 = 1;
pub const MARTINA_COL: u32 = 3;
pub const JOSEPHINA_COL: u32 = 5;
