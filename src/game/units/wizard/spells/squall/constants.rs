//! Squall spell constants.

use crate::game::units::wizard::components::PrimedSpell;

/// Mana cost to cast Squall.
pub const MANA_COST: f32 = 40.0;

/// Cast time for Squall (seconds) - short cast like Guardian Circle.
pub const CAST_TIME: f32 = 0.5;

/// Radius of the storm circle where ice projectiles will rain down.
pub const STORM_RADIUS: f32 = 400.0;

/// Y position for the circle indicator on the ground.
pub const CIRCLE_Y_POSITION: f32 = 0.1;

/// Time between ice projectile spawns (seconds).
pub const ICE_SPAWN_INTERVAL: f32 = 0.2;

/// Height above battlefield where ice projectiles spawn (above camera view).
pub const ICE_SPAWN_HEIGHT: f32 = 2000.0;

/// Downward velocity when ice projectiles spawn.
pub const ICE_INITIAL_VELOCITY: f32 = -100.0;

/// Gravity acceleration applied to falling ice projectiles.
pub const ICE_GRAVITY: f32 = -500.0;

/// Radius of the ice projectile for collision detection.
pub const ICE_PROJECTILE_RADIUS: f32 = 5.0;

/// Visual radius of the ice projectile mesh.
pub const ICE_PROJECTILE_MESH_RADIUS: f32 = 8.0;

/// Frost damage dealt by each ice explosion.
pub const FROST_DAMAGE: f32 = 10.0;

/// Radius of the ice explosion damage area.
pub const EXPLOSION_RADIUS: f32 = 40.0;

/// Lifetime of the ice explosion visual effect (seconds).
pub const EXPLOSION_LIFETIME: f32 = 0.4;

/// Duration of the frost slow effect applied to units (seconds).
pub const FROST_SLOW_DURATION: f32 = 2.5;

/// Movement speed reduction percentage from frost slow (negative value).
/// -0.4 = 40% speed reduction
pub const FROST_SLOW_MODIFIER: f32 = -0.4;

/// Growth time for explosion visual (seconds).
pub const EXPLOSION_GROWTH_TIME: f32 = 0.15;

/// Primed Squall spell configuration.
pub const PRIMED_SQUALL: PrimedSpell = PrimedSpell {
    spell: crate::game::units::wizard::components::Spell::Squall,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};
