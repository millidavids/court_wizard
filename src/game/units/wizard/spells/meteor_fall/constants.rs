//! Meteor Fall spell constants.

use crate::game::units::wizard::components::PrimedSpell;

/// Primed Meteor Fall spell configuration.
pub const PRIMED_METEOR_FALL: PrimedSpell = PrimedSpell {
    spell: crate::game::units::wizard::components::Spell::MeteorFall,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time for Meteor Fall (seconds).
pub const CAST_TIME: f32 = 1.0;

/// Mana cost to cast Meteor Fall.
pub const MANA_COST: f32 = 60.0;

/// Radius of the storm circle where meteors will rain down.
pub const STORM_RADIUS: f32 = 300.0;

/// Y position for the circle indicator on the ground.
pub const CIRCLE_Y_POSITION: f32 = 0.1;

/// Time between meteor spawns (seconds).
pub const METEOR_SPAWN_INTERVAL: f32 = 0.8;

/// Height above battlefield where meteors spawn (above camera view).
pub const METEOR_SPAWN_HEIGHT: f32 = 2000.0;

/// Downward velocity when meteors spawn.
pub const METEOR_INITIAL_VELOCITY: f32 = -100.0;

/// Gravity acceleration applied to falling meteors.
pub const METEOR_GRAVITY: f32 = -500.0;

/// Visual radius of the meteor projectile mesh.
pub const METEOR_MESH_RADIUS: f32 = 12.0;

/// Fire damage dealt by each meteor explosion.
pub const METEOR_DAMAGE: f32 = 25.0;

/// Radius of the meteor explosion damage area.
pub const EXPLOSION_RADIUS: f32 = 50.0;

/// Lifetime of the explosion visual effect (seconds).
pub const EXPLOSION_LIFETIME: f32 = 0.4;

/// Growth time for explosion visual (seconds).
pub const EXPLOSION_GROWTH_TIME: f32 = 0.15;

/// Radius of the persistent ground fire zone.
pub const GROUND_FIRE_RADIUS: f32 = 30.0;

/// Duration of the ground fire hazard (seconds).
pub const GROUND_FIRE_DURATION: f32 = 8.0;

/// Damage per tick for ground fire.
pub const GROUND_FIRE_DAMAGE: f32 = 4.0;

/// Time between ground fire damage ticks (seconds).
pub const GROUND_FIRE_TICK: f32 = 0.5;

/// Duration of the fade-out effect before ground fire expires (seconds).
pub const GROUND_FIRE_FADE_DURATION: f32 = 2.0;
