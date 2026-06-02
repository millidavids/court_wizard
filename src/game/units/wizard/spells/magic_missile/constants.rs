//! Magic missile spell constants.
//!
//! Contains all hardcoded values for magic missile behavior.

use crate::game::units::DamageType;
use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Magic Missile.
pub const PRIMED_MAGIC_MISSILE: PrimedSpell = PrimedSpell {
    spell: Spell::MagicMissile,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Height offset above spell origin for magic missile spawn.
pub const SPAWN_HEIGHT_OFFSET: f32 = 0.0;

/// Minimum horizontal velocity for magic missile spawn.
pub const HORIZONTAL_VEL_MIN: f32 = -2000.0;

/// Maximum horizontal velocity for magic missile spawn.
pub const HORIZONTAL_VEL_MAX: f32 = 2000.0;

/// Minimum vertical velocity for magic missile spawn.
pub const VERTICAL_VEL_MIN: f32 = -2000.0;

/// Maximum vertical velocity for magic missile spawn.
pub const VERTICAL_VEL_MAX: f32 = 2000.0;

/// Minimum camera arc speed for magic missiles.
pub const CAMERA_ARC_SPEED_MIN: f32 = 3200.0;

/// Maximum camera arc speed for magic missiles.
pub const CAMERA_ARC_SPEED_MAX: f32 = 4800.0;

/// Base homing strength for magic missiles.
pub const BASE_HOMING_STRENGTH: f32 = 400.0;

/// Base speed for magic missiles.
pub const BASE_SPEED: f32 = 600.0;

/// Final speed multiplier for magic missiles after ramp-up.
pub const FINAL_SPEED_MULTIPLIER: f32 = 3.0;

/// Speed ramp multiplier for magic missiles.
pub const SPEED_RAMP_MULTIPLIER: f32 = 2.0;

/// Time for magic missile homing to ramp up to perfect tracking (seconds).
pub const PERFECT_TRACKING_TIME: f32 = 5.0;

/// Homing strength multiplier over perfect tracking time.
pub const HOMING_RAMP_MULTIPLIER: f32 = 19.0;

/// Minimum speed for magic missiles during proximity slowdown.
pub const MIN_PROXIMITY_SPEED: f32 = 300.0;

/// Distance threshold for magic missile proximity slowdown.
pub const SLOWDOWN_DISTANCE: f32 = 300.0;

/// Damage dealt by each magic missile.
pub const DAMAGE: f32 = 24.0;

/// Type of damage dealt by magic missiles.
pub const DAMAGE_TYPE: DamageType = DamageType::Force;

/// Collision radius for magic missiles.
pub const COLLISION_RADIUS: f32 = 10.0;

/// Wobble frequency for magic missiles.
pub const WOBBLE_FREQUENCY: f32 = 3.0;

/// Wobble amplitude for magic missiles.
pub const WOBBLE_AMPLITUDE: f32 = 30.0;

/// Wobble Y-axis frequency multiplier.
pub const WOBBLE_Y_FREQ_MULTIPLIER: f32 = 1.3;

/// Wobble Z-axis frequency multiplier.
pub const WOBBLE_Z_FREQ_MULTIPLIER: f32 = 0.7;

/// Wobble Y-axis amplitude multiplier.
pub const WOBBLE_Y_AMPLITUDE_MULTIPLIER: f32 = 0.5;

/// Mana cost per volley of magic missiles.
pub const MANA_COST: f32 = 8.0;

/// Cast time for magic missile in seconds (0 = instant cast).
pub const CAST_TIME: f32 = 0.0;

/// Number of missiles spawned per cast.
pub const MISSILES_PER_CAST: u32 = 3;

/// Cooldown between casts in seconds.
pub const COOLDOWN: f32 = 1.0;

/// Display name for the Arcane Barrage talent / concentration spell.
pub const ARCANE_BARRAGE_NAME: &str = "Arcane Barrage";

/// Base seconds between Arcane Barrage volleys. The Swift Salvo talent scales
/// this down via `cooldown_mult` (×0.75 → ~1.875s).
pub const ARCANE_BARRAGE_INTERVAL: f32 = 2.5;

/// Power for inverse distance weighting in cursor targeting.
/// Higher values = stronger preference for targets near cursor.
/// 2.0 = inverse square, 1.0 = linear inverse
pub const CURSOR_TARGETING_WEIGHT_POWER: i32 = 2;
