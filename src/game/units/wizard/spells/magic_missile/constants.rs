//! Magic missile spell constants.
//!
//! Contains all hardcoded values for magic missile behavior.

/// Height offset above wizard for magic missile spawn.
pub const SPAWN_HEIGHT_OFFSET: f32 = 100.0;

/// Minimum horizontal velocity for magic missile spawn.
pub const HORIZONTAL_VEL_MIN: f32 = -200.0;

/// Maximum horizontal velocity for magic missile spawn.
pub const HORIZONTAL_VEL_MAX: f32 = 200.0;

/// Minimum vertical velocity for magic missile spawn.
pub const VERTICAL_VEL_MIN: f32 = 300.0;

/// Maximum vertical velocity for magic missile spawn.
pub const VERTICAL_VEL_MAX: f32 = 500.0;

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
pub const DAMAGE: f32 = 50.0;

/// Collision radius for magic missiles.
pub const COLLISION_RADIUS: f32 = 10.0;

/// Maximum distance before magic missiles despawn.
pub const MAX_DISTANCE: f32 = 10000.0;

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

/// Mana cost for casting a magic missile.
pub const MANA_COST: f32 = 10.0;

/// Cast time for magic missile in seconds.
pub const CAST_TIME: f32 = 1.0;

/// Initial interval between channeled magic missiles (in seconds).
pub const INITIAL_CHANNEL_INTERVAL: f32 = 0.5;

/// Minimum interval between channeled magic missiles (in seconds).
pub const MIN_CHANNEL_INTERVAL: f32 = 0.05;

/// Time to ramp from initial to minimum channel interval (in seconds).
pub const CHANNEL_RAMP_TIME: f32 = 5.0;
