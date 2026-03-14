use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_GREASE: PrimedSpell = PrimedSpell {
    spell: Spell::Grease,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 1.5;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 150.0;
pub const SLOW_MODIFIER: f32 = -0.4;
pub const SLOW_DURATION: f32 = 1.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const ZONE_DURATION: f32 = 20.0;
pub const IGNITE_DAMAGE: f32 = 0.0;
pub const IGNITE_BURN_DAMAGE: f32 = 1.0;
pub const IGNITE_BURN_TICK: f32 = 0.5;
pub const CIRCLE_Y_POSITION: f32 = 2.0;
pub const FADE_DURATION: f32 = 2.0;
/// Duration in seconds for grease zone to grow from 0 to full radius on spawn
pub const GROW_DURATION: f32 = 0.4;

/// Max Y height a fire source can be at to ignite grease (filters out aerial spells)
pub const IGNITION_HEIGHT_THRESHOLD: f32 = 15.0;
/// Time in seconds for fire to spread across the full grease radius
pub const FIRE_SPREAD_DURATION: f32 = 1.0;
/// Fraction of zone radius for initial burst damage at ignition point
pub const IGNITION_BURST_RADIUS_FRACTION: f32 = 0.3;
/// Interval between smoke wisp spawns for burning grease
pub const FIRE_SMOKE_INTERVAL: f32 = 0.25;

// === Talent Constants ===

// Tier 1: Extra Slippery — slow effect multiplier
pub(super) const EXTRA_SLIPPERY_SLOW_MULT: f32 = 1.3;

// Tier 1: Wider Slick — zone radius multiplier
pub(super) const WIDER_SLICK_RADIUS_MULT: f32 = 1.4;

// Tier 1: Volatile Mixture — ignited burn damage multiplier
pub(super) const VOLATILE_MIXTURE_BURN_MULT: f32 = 1.5;

// Tier 2: Slip and Fall — chance to stun on zone entry
pub(super) const SLIP_AND_FALL_CHANCE: f32 = 0.3;
/// Duration of the prone stun from Slip and Fall
pub(super) const SLIP_AND_FALL_STUN_DURATION: f32 = 1.5;

// Tier 2: Oil Slick — spell vulnerability increase while in zone
pub(super) const OIL_SLICK_VULNERABILITY: f32 = 0.2;

// Tier 3: Chain Combustion — extra range multiplier for chain-ignition between zones
pub(super) const CHAIN_COMBUSTION_RANGE_MULT: f32 = 2.0;

// Tier 3: Grease Geyser — launch parameters
pub(super) const GEYSER_LAUNCH_VELOCITY: f32 = 200.0;
pub(super) const GEYSER_GRAVITY: f32 = 120.0;
/// Duration of root applied during geyser launch (keeps unit in place while airborne)
pub(super) const GEYSER_ROOT_DURATION: f32 = 3.5;

// Tier 3: Endless Oil — regeneration time after fire burns out
pub(super) const ENDLESS_OIL_REGEN_DURATION: f32 = 10.0;
/// How much extra duration to add when regenerating (so the zone has time to be slippery again)
pub(super) const ENDLESS_OIL_EXTRA_DURATION: f32 = 15.0;

// === Visual Constants ===

/// Base color for grease zone mesh (RGBA, rendered opaque via AlphaMode::Mask).
pub(super) const GREASE_COLOR: (f32, f32, f32, f32) = (0.25, 0.22, 0.08, 0.85);

// ── Grease zone VFX ─────────────────────────────────────────────────
/// Time between fume wisp batch spawns (seconds).
pub(super) const FUME_SPAWN_INTERVAL: f32 = 0.15;
/// Number of fume wisps per spawn batch.
pub(super) const FUME_COUNT_PER_SPAWN: usize = 2;
/// How long each fume wisp lives (seconds).
pub(super) const FUME_LIFETIME: f32 = 1.5;
/// Base size of fume wisp particles.
pub(super) const FUME_SIZE: f32 = 8.0;
/// Upward drift speed of fume wisps.
pub(super) const FUME_RISE_SPEED: f32 = 15.0;
/// Lateral spread speed of fume wisps.
pub(super) const FUME_SPREAD_SPEED: f32 = 8.0;

/// Time between bubble spawns (seconds).
pub(super) const BUBBLE_SPAWN_INTERVAL: f32 = 0.2;
/// Bubble lifetime range: min and max (seconds).
pub(super) const BUBBLE_LIFETIME_MIN: f32 = 0.4;
pub(super) const BUBBLE_LIFETIME_MAX: f32 = 0.9;
/// Bubble size range.
pub(super) const BUBBLE_SIZE_MIN: f32 = 3.0;
pub(super) const BUBBLE_SIZE_MAX: f32 = 7.0;
/// Bubble upward rise speed.
pub(super) const BUBBLE_RISE_SPEED: f32 = 12.0;

/// Time between splatter spawns (seconds).
pub(super) const SPLATTER_SPAWN_INTERVAL: f32 = 0.25;
/// Splatter lifetime (seconds).
pub(super) const SPLATTER_LIFETIME: f32 = 1.0;
/// Splatter particle size.
pub(super) const SPLATTER_SIZE: f32 = 6.0;
