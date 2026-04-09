//! Squall spell constants.

use crate::game::units::wizard::components::PrimedSpell;

/// Mana cost to cast Squall.
pub const MANA_COST: f32 = 30.0;

/// Cast time for Squall (seconds) - short cast like Guardian Circle.
pub const CAST_TIME: f32 = 0.5;

/// Radius of the storm circle where ice projectiles will rain down.
pub const STORM_RADIUS: f32 = 400.0;

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

/// Growth time for explosion visual (seconds).
pub const EXPLOSION_GROWTH_TIME: f32 = 0.15;

// ── Frost accumulation ──────────────────────────────────────────────────
/// Frost accumulation added per ice hit (0.0–1.0 scale).
pub const FROST_PER_HIT: f32 = 0.3;
/// Frost accumulation per hit with Permafrost talent.
pub const PERMAFROST_FROST_PER_HIT: f32 = 0.6;
/// Seconds after last hit before frost starts decaying.
pub const FROST_DECAY_DELAY: f32 = 1.5;
/// Frost decay rate per second (after delay expires).
pub const FROST_DECAY_RATE: f32 = 0.15;
/// Maximum slow at full frost before freeze (-0.5 = 50% slow).
pub const FROST_MAX_SLOW: f32 = -0.5;
/// Duration of the freeze when frost reaches 1.0.
pub const FROST_FREEZE_DURATION: f32 = 2.0;

// === Talent Constants ===

// Tier 1: Bitter Cold
pub const BITTER_COLD_DAMAGE_MULT: f32 = 1.3;

// Tier 1: Howling Winds
pub const HOWLING_WINDS_RADIUS_MULT: f32 = 1.3;

// Tier 1: Freezing Rain
/// Spawn interval multiplier (lower = faster spawning).
pub const FREEZING_RAIN_SPAWN_MULT: f32 = 0.6;
pub const FREEZING_RAIN_DAMAGE_MULT: f32 = 0.8;

// Tier 2: Permafrost
/// Duration of the freeze-solid effect with Permafrost talent (seconds).
pub const PERMAFROST_FREEZE_DURATION: f32 = 3.0;

// Tier 2: Hailstones
/// Chance for any given projectile to be a hailstone (0.0–1.0).
pub const HAILSTONE_CHANCE: f32 = 0.25;
/// Damage multiplier for hailstone projectiles.
pub const HAILSTONE_DAMAGE_MULT: f32 = 3.0;
/// Visual scale multiplier for hailstone projectiles.
pub const HAILSTONE_MESH_SCALE: f32 = 2.0;

// Tier 2: Sleet Storm
/// Evasion chance applied to enemies inside the storm radius.
pub const SLEET_STORM_EVASION_CHANCE: f32 = 0.4;
/// Duration of the evasion debuff after leaving the storm.
pub const SLEET_STORM_EVASION_DURATION: f32 = 1.0;

// Tier 3: Absolute Zero
/// Mana drained per second while Absolute Zero is channeling.
pub const ABSOLUTE_ZERO_MANA_PER_SEC: f32 = 12.0;
/// Stacking slow added per frame (0.005 = 0.5% per frame).
pub const ABSOLUTE_ZERO_SLOW_PER_FRAME: f32 = 0.005;
/// Maximum slow that can stack from Absolute Zero (0.9 = 90% slow).
pub const ABSOLUTE_ZERO_MAX_SLOW: f32 = 0.9;
/// Damage per second dealt to units inside the Absolute Zero zone.
pub const ABSOLUTE_ZERO_DPS: f32 = 5.0;
/// Duration the stacking slow persists after leaving the zone or channeling stops (seconds).
pub const ABSOLUTE_ZERO_SLOW_DECAY_TIME: f32 = 3.0;

// Tier 3: Blizzard
/// Speed at which the storm follows the cursor (units/second).
pub const BLIZZARD_FOLLOW_SPEED: f32 = 150.0;

// Tier 3: Ice Age
/// Slow modifier for frozen ground (-0.3 = 30% slow).
pub const ICE_AGE_SLOW_MODIFIER: f32 = -0.3;
/// Duration the slow lasts after leaving frozen ground (seconds).
pub const ICE_AGE_SLOW_DURATION: f32 = 2.0;
/// How long frozen ground persists after the storm ends (seconds).
pub const ICE_AGE_GROUND_DURATION: f32 = 15.0;
/// Radius of each frozen ground patch.
pub const ICE_AGE_PATCH_RADIUS: f32 = 60.0;

// === Snow VFX Constants ===

/// Time between snow particle spawns (seconds).
pub const SNOW_SPAWN_INTERVAL: f32 = 0.03;
/// Number of snow particles to spawn per batch.
pub const SNOW_BATCH_SIZE: u32 = 2;
/// Lifetime of snow particles (seconds).
pub const SNOW_LIFETIME: f32 = 2.5;
/// Base visual size of snow particles.
pub const SNOW_BASE_SIZE: f32 = 6.0;
/// Height range for snow particle spawning (above battlefield).
pub const SNOW_MIN_HEIGHT: f32 = 30.0;
pub const SNOW_MAX_HEIGHT: f32 = 120.0;
/// Swirl speed (tangential velocity around storm center).
pub const SNOW_SWIRL_SPEED: f32 = 40.0;
/// Lateral sway amplitude.
pub const SNOW_SWAY_AMPLITUDE: f32 = 6.0;
/// Lateral sway frequency (Hz).
pub const SNOW_SWAY_FREQUENCY: f32 = 1.2;
/// Downward drift speed.
pub const SNOW_DRIFT_SPEED: f32 = 15.0;

/// Primed Squall spell configuration.
pub const PRIMED_SQUALL: PrimedSpell = PrimedSpell {
    spell: crate::game::units::wizard::components::Spell::Squall,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};
