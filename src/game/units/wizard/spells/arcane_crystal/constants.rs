//! Arcane Crystal spell constants.

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Arcane Crystal.
pub const PRIMED_ARCANE_CRYSTAL: PrimedSpell = PrimedSpell {
    spell: Spell::ArcaneCrystal,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// ===== Casting & Mana =====

/// Cast time before crystal spawns (in seconds).
pub const CAST_TIME: f32 = 2.0;

/// Mana cost to place the crystal.
pub const MANA_COST: f32 = 35.0;

// ===== Crystal Properties =====

/// Total lifetime of the crystal (in seconds).
pub const CRYSTAL_DURATION: f32 = 25.0;

/// Range within which the crystal targets enemies and limits projectiles.
pub const CRYSTAL_RANGE: f32 = 500.0;

/// Collision radius for detecting incoming spell hits.
pub const CRYSTAL_COLLISION_RADIUS: f32 = 20.0;

/// Visual height of the crystal mesh.
pub const CRYSTAL_HEIGHT: f32 = 35.0;

// ===== Scaling =====

/// Global damage scale for all crystal emissions (fraction of original spell damage).
pub const DAMAGE_SCALE: f32 = 0.5;

/// Global size scale for crystal emissions (fraction of original spell size).
pub const SIZE_SCALE: f32 = 0.5;

/// Speed scale for crystal projectiles (fraction of original speed).
pub const SPEED_SCALE: f32 = 0.7;

/// Number of mini fireballs emitted per absorbed fireball.
pub const MINI_FB_COUNT: usize = 5;

/// Duration of crystal-emitted beams (seconds).
pub const BEAM_DURATION: f32 = 0.5;

/// Number of lesser beams emitted by the crystal for disintegrate and finger of death.
pub const BEAM_COUNT: usize = 5;

/// Damage scale for crystal beams (50% of original).
pub const BEAM_DAMAGE_SCALE: f32 = 0.5;

/// Half-angle of the forked fan spread for crystal beams (radians, ~7.5 degrees).
pub const FORKED_FAN_HALF_ANGLE: f32 = 0.13;

/// Number of targets for crystal lightning arcs.
pub const LIGHTNING_ARC_COUNT: usize = 3;

/// Height above crystal from which mini meteors are launched.
pub const MINI_METEOR_SPAWN_HEIGHT: f32 = 200.0;

// ===== Visual =====

/// Y position of the range indicator circle.
pub const RANGE_INDICATOR_Y: f32 = 1.0;

/// Rotation speed of the crystal (radians/sec).
pub const ROTATION_SPEED: f32 = 0.5;

/// Scale factor for absorption pulse animation.
pub const PULSE_SCALE: f32 = 1.3;

/// Duration of absorption pulse animation.
pub const PULSE_DURATION: f32 = 0.15;

// ===== Mini Missiles =====

/// Number of mini magic missiles emitted per absorbed missile.
pub const MINI_MISSILE_COUNT: usize = 5;

/// Pre-advance time_alive on crystal-spawned missiles so homing is nearly perfect immediately.
/// Magic missile PERFECT_TRACKING_TIME is 5.0s; this skips most of the ramp.
pub const MINI_MISSILE_HOMING_ADVANCE: f32 = 4.0;

// ===== Auto-Cast =====

/// Base reference damage for auto-cast interval scaling.
/// Spells with this damage get an interval of AUTO_CAST_BASE_INTERVAL.
pub const AUTO_CAST_REFERENCE_DAMAGE: f32 = 20.0;

/// Base auto-cast interval (seconds) for a spell dealing REFERENCE_DAMAGE.
pub const AUTO_CAST_BASE_INTERVAL: f32 = 3.0;

/// Minimum auto-cast interval (floor, seconds).
pub const AUTO_CAST_MIN_INTERVAL: f32 = 1.5;

/// Maximum auto-cast interval (ceiling, seconds).
pub const AUTO_CAST_MAX_INTERVAL: f32 = 20.0;
