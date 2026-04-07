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

/// Interval between fire particle spawns for ground fire pools (seconds).
pub const GROUND_FIRE_SMOKE_INTERVAL: f32 = 0.25;

/// Number of procedural fire particle batches per ground fire pool spawn tick.
/// Kept low (vs fireball's 9) because many meteor pools can overlap simultaneously.
pub const GROUND_FIRE_PARTICLE_COUNT: usize = 3;

// === Talent constants ===

// Tier 1
/// Dense Barrage: meteor spawn rate multiplier.
pub const DENSE_BARRAGE_SPAWN_RATE_MULT: f32 = 1.3;
/// Scorching Impact: explosion and ground fire damage multiplier.
pub const SCORCHING_IMPACT_DAMAGE_MULT: f32 = 1.3;
/// Wide Devastation: storm radius multiplier.
pub const WIDE_DEVASTATION_RADIUS_MULT: f32 = 1.3;

// Tier 2
/// Molten Core: ground fire duration multiplier.
pub const MOLTEN_CORE_DURATION_MULT: f32 = 2.0;
/// Molten Core: ground fire damage multiplier.
pub const MOLTEN_CORE_DAMAGE_MULT: f32 = 1.5;
/// Tracking Meteors: horizontal acceleration toward nearest enemy.
pub const TRACKING_FORCE: f32 = 150.0;
/// Aftershock: radius for knockback effect.
pub const AFTERSHOCK_RADIUS: f32 = 80.0;
/// Aftershock: knockback speed.
pub const AFTERSHOCK_KNOCKBACK_SPEED: f32 = 200.0;
/// Aftershock: knockback duration.
pub const AFTERSHOCK_KNOCKBACK_DURATION: f32 = 0.3;
/// Aftershock: bonus AoE damage.
pub const AFTERSHOCK_DAMAGE: f32 = 10.0;

// Tier 3
/// Extinction Event: delay before the massive meteor strikes (seconds).
pub const EXTINCTION_DELAY: f32 = 5.0;
/// Extinction Event: total storm duration (delay + time for meteor to land).
pub const EXTINCTION_STORM_DURATION: f32 = 8.0;
/// Extinction Event: damage of the massive meteor.
pub const EXTINCTION_DAMAGE: f32 = 100.0;
/// Extinction Event: visual mesh radius.
pub const EXTINCTION_MESH_RADIUS: f32 = 50.0;
/// Volcanic Eruption: base eruption damage.
pub const VOLCANIC_ERUPTION_BASE_DAMAGE: f32 = 15.0;
/// Volcanic Eruption: bonus damage per stack.
pub const VOLCANIC_ERUPTION_STACK_BONUS: f32 = 5.0;
/// Volcanic Eruption: radius for detecting nearby ground fire.
pub const VOLCANIC_ERUPTION_RADIUS: f32 = 50.0;
/// Meteor Shower: spawn rate multiplier (3x meteors).
pub const METEOR_SHOWER_SPAWN_RATE_MULT: f32 = 3.0;
/// Meteor Shower: damage multiplier (40% damage).
pub const METEOR_SHOWER_DAMAGE_MULT: f32 = 0.4;
/// Meteor Shower: explosion/ground fire radius multiplier (60%).
pub const METEOR_SHOWER_RADIUS_MULT: f32 = 0.6;
/// Meteor Shower: mana cost multiplier (50%).
pub const METEOR_SHOWER_MANA_MULT: f32 = 0.5;
/// Meteor Shower: visual mesh multiplier (60%).
pub const METEOR_SHOWER_MESH_MULT: f32 = 0.6;
