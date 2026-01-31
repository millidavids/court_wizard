//! Fireball spell constants.
//!
//! Contains all hardcoded values for fireball behavior.

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Fireball.
pub const PRIMED_FIREBALL: PrimedSpell = PrimedSpell {
    spell: Spell::Fireball,
    cast_time: CAST_TIME,
};

/// Height offset above wizard for fireball spawn.
pub const SPAWN_HEIGHT_OFFSET: f32 = 100.0;

/// Cast time for fireball in seconds.
pub const CAST_TIME: f32 = 3.0;

/// Mana cost for casting a fireball.
pub const MANA_COST: f32 = 30.0;

/// Speed of the fireball projectile in units per second.
pub const PROJECTILE_SPEED: f32 = 3000.0;

/// Collision radius for the fireball projectile.
pub const PROJECTILE_COLLISION_RADIUS: f32 = 15.0;

/// Maximum radius of the explosion in units.
pub const EXPLOSION_RADIUS: f32 = 100.0;

/// Duration of the explosion animation in seconds.
pub const EXPLOSION_DURATION: f32 = 0.4;

/// Interval between damage ticks in seconds.
pub const DAMAGE_TICK_INTERVAL: f32 = 0.05;

/// Total damage dealt to a unit that stays in the explosion for the full duration.
pub const TOTAL_DAMAGE: f32 = 25.0;

/// Damage dealt per tick to units in the explosion.
/// Calculated as TOTAL_DAMAGE / (EXPLOSION_DURATION / DAMAGE_TICK_INTERVAL)
pub const DAMAGE_PER_TICK: f32 = TOTAL_DAMAGE / (EXPLOSION_DURATION / DAMAGE_TICK_INTERVAL);

// ===== Residual Fire Constants =====

/// Radius of the residual fire area.
pub const RESIDUAL_DAMAGE_RADIUS: f32 = 100.0;

/// Damage dealt per tick to units in the residual fire.
pub const RESIDUAL_DAMAGE_PER_TICK: f32 = 3.75;

/// Time between residual fire damage ticks (seconds).
pub const RESIDUAL_TICK_INTERVAL: f32 = 0.25;

/// Total duration of the residual fire effect (seconds).
pub const RESIDUAL_DURATION: f32 = 5.0;

/// Duration of the fade-out at the end of the residual fire (seconds).
pub const RESIDUAL_FADE_DURATION: f32 = 1.0;
