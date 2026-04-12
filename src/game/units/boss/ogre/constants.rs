use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// Visual appearance
pub const OGRE_COLOR: Color = Color::WHITE;
pub const OGRE_ENRAGE_1_COLOR: Color = Color::srgb(1.0, 0.85, 0.85);
pub const OGRE_ENRAGE_2_COLOR: Color = Color::srgb(1.0, 0.7, 0.7);
pub const OGRE_ENRAGE_3_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
pub const OGRE_ELLIPSE_WIDTH: f32 = 40.0 * UNIT_SCALE;
pub const OGRE_RADIUS: f32 = 20.0 * UNIT_SCALE;
/// Hitbox extends from ground to the top of the ogre sprite.
pub const OGRE_HITBOX_HEIGHT: f32 = OGRE_SPRITE_HEIGHT - OGRE_SPRITE_Y_OFFSET;

// Movement
pub const OGRE_MOVEMENT_SPEED: f32 = 320.0;

// Combat
pub const OGRE_HEALTH: f32 = 6000.0;
pub const OGRE_DAMAGE_MULTIPLIER: f32 = -0.5;
pub const OGRE_ATTACK_DAMAGE: f32 = 30.0;
pub const OGRE_ATTACK_COOLDOWN: f32 = 1.0;
/// Extra melee attack reach beyond standard hitbox-based range (world units).
pub const OGRE_MELEE_RANGE_BONUS: f32 = 80.0;
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
pub const OGRE_CHARGE_TARGET_MAX_DISTANCE: f32 = OGRE_CHARGE_TARGET_MIN_DISTANCE * 1.5;
pub const OGRE_CHARGE_TELEGRAPH_DURATION: f32 = 3.0;
pub const OGRE_CHARGE_SPEED: f32 = 1200.0;
/// Charge travels further than the targeting range so the ogre overshoots the target.
pub const OGRE_CHARGE_MAX_DISTANCE: f32 = OGRE_CHARGE_TARGET_MAX_DISTANCE * 1.2;
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
pub const OGRE_CHARGE_LINE_COLOR: Color = Color::srgba(0.8, 0.1, 0.05, 0.25);
pub const OGRE_CHARGE_FILL_BASE_COLOR: Color = Color::srgba(0.6, 0.05, 0.02, 0.15);
pub const OGRE_CHARGE_PULSE_FREQUENCY: f32 = 2.5;
/// Peak emissive intensity at full charge (linear RGB scale).
pub const OGRE_CHARGE_EMISSIVE_MAX: f32 = 8.0;

/// Fraction of melee damage the ogre actually takes (0.15 = 85% reduction).
/// Spells deal full damage, making magic the primary way to hurt him.
pub const OGRE_MELEE_DAMAGE_REDUCTION: f32 = 0.15;

// Sprite sheet configuration (all sheets are 512x512, 4 columns x 4 rows, 128px frames)
pub const OGRE_SPRITE_COLUMNS: usize = 4;
pub const OGRE_FRAME_UV: Vec2 = Vec2::new(0.25, 0.25);

/// Walking sheet row order: Right(0), Forward(1), Left(2), Away(3).
pub const OGRE_WALKING_DIRECTION_ROWS: [usize; 4] = [3, 1, 2, 0];
/// Attacking sheet row order: Forward(0), Away(1), Left(2), Right(3).
pub const OGRE_ATTACKING_DIRECTION_ROWS: [usize; 4] = [1, 0, 2, 3];
/// Throwing sheet row order: Forward(0), Right(1), Left(2), Away(3).
pub const OGRE_THROWING_DIRECTION_ROWS: [usize; 4] = [3, 0, 2, 1];

// Sprite display dimensions (world units)
pub const OGRE_SPRITE_WIDTH: f32 = 120.0 * UNIT_SCALE;
pub const OGRE_SPRITE_HEIGHT: f32 = 160.0 * UNIT_SCALE;

/// Vertical offset to align the ogre sprite's feet with the ground plane.
/// The sprite has empty space above the ogre in each frame.
pub const OGRE_SPRITE_Y_OFFSET: f32 = 90.0;

// Charge telegraph visual effects
pub const OGRE_CHARGE_VIBRATION_AMPLITUDE: f32 = 6.0;
pub const OGRE_CHARGE_VIBRATION_FREQ_X: f32 = 17.0;
pub const OGRE_CHARGE_VIBRATION_FREQ_Z: f32 = 23.0;
pub const OGRE_CHARGE_FLASH_FREQUENCY: f32 = 4.0;
pub const OGRE_CHARGE_FLASH_COLOR: Color = Color::srgb(1.0, 0.15, 0.1);
