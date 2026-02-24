//! Black Hole spell constants.

use crate::game::units::DamageType;
use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Black Hole.
pub const PRIMED_BLACK_HOLE: PrimedSpell = PrimedSpell {
    spell: Spell::BlackHole,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// ===== Casting & Mana =====

/// Cast time before black hole spawns (in seconds).
pub const CAST_TIME: f32 = 20.0;

/// Upfront mana cost to cast the spell.
pub const MANA_COST: f32 = 100.0;

// ===== Lifetime & Growth =====

/// Total lifetime of the black hole (in seconds).
pub const LIFETIME: f32 = 20.0;

/// Time for black hole to grow from zero to max radius (in seconds).
pub const GROWTH_TIME: f32 = 5.0;

/// Maximum radius of the black hole sphere (in units).
pub const MAX_RADIUS: f32 = 50.0;

/// Height offset from ground where black hole spawns.
/// Matches infantry unit Y position (hitbox.height / 2.0 + 1.0 = 25.0 / 2.0 + 1.0 = 13.5).
pub const BLACK_HOLE_HEIGHT: f32 = 13.5;

// ===== Gravitational Physics =====

/// Base gravitational pull strength (units/second² at time=0).
pub const BASE_GRAVITY_STRENGTH: f32 = 20000000.0;

/// Maximum gravitational pull strength (units/second² at full ramp).
pub const MAX_GRAVITY_STRENGTH: f32 = 100000000.0;

/// Time for gravity to ramp from base to max strength (in seconds).
pub const GRAVITY_RAMP_TIME: f32 = 5.0;

/// Maximum distance at which gravitational effects apply (in units).
pub const GRAVITY_RANGE: f32 = 500.0;

/// Maximum force clamp to prevent excessive acceleration at close range.
pub const MAX_FORCE_CLAMP: f32 = 2500.0;

// ===== Damage =====

/// Base damage per tick to units touching the sphere.
pub const BASE_DAMAGE_PER_TICK: f32 = 8.0;

/// Type of damage dealt by black holes.
pub const DAMAGE_TYPE: DamageType = DamageType::Force;

/// Time between damage ticks (in seconds).
pub const DAMAGE_INTERVAL: f32 = 0.2;

/// Time for damage to ramp up to maximum for a unit inside (in seconds).
pub const DAMAGE_RAMP_TIME: f32 = 3.0;

/// Maximum damage multiplier for units that have been inside for a while.
pub const MAX_DAMAGE_MULTIPLIER: f32 = 3.0;

// ===== Visual =====

/// Amplitude of the vibration effect (in units).
pub const VIBRATION_AMPLITUDE: f32 = 2.0;

/// Frequency of the vibration effect (cycles per second).
pub const VIBRATION_FREQUENCY: f32 = 8.0;
