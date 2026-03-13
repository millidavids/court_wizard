use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_SPIKE_GROWTH: PrimedSpell = PrimedSpell {
    spell: Spell::SpikeGrowth,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 2.5;
pub const MANA_COST: f32 = 30.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const DAMAGE_PER_TICK: f32 = 3.0;
pub const TICK_INTERVAL: f32 = 0.5;
pub const ZONE_DURATION: f32 = 15.0;
pub const SLOW_MODIFIER: f32 = -0.3;
pub const SLOW_DURATION: f32 = 2.0;
pub const CIRCLE_Y_POSITION: f32 = 2.0;

// --- Tier 1 talent constants ---

/// Wider Zone: radius multiplier.
pub(super) const WIDER_ZONE_RADIUS_MULT: f32 = 1.4;

/// Sharper Spikes: damage per tick multiplier.
pub(super) const SHARPER_SPIKES_DAMAGE_MULT: f32 = 1.6;

/// Quick Bloom: cast time multiplier.
pub(super) const QUICK_BLOOM_CAST_TIME_MULT: f32 = 0.6;
/// Quick Bloom: mana cost multiplier.
pub(super) const QUICK_BLOOM_MANA_MULT: f32 = 0.7;

// --- Tier 2 talent constants ---

/// Thorn Maze: slow modifier (replaces base slow).
pub(super) const THORN_MAZE_SLOW_MODIFIER: f32 = -0.6;
/// Thorn Maze: hazard cost multiplier for pathfinding avoidance.
pub(super) const THORN_MAZE_HAZARD_MULT: f32 = 2.0;

/// Poisoned Spikes: lingering poison DPS after leaving zone.
pub(super) const POISONED_SPIKES_LINGER_DPS: f32 = 2.0;
/// Poisoned Spikes: lingering poison duration (seconds).
pub(super) const POISONED_SPIKES_LINGER_DURATION: f32 = 4.0;
/// Poisoned Spikes: lingering tick interval (seconds).
pub(super) const POISONED_SPIKES_LINGER_TICK_INTERVAL: f32 = 0.5;

/// Quicksand: time in zone before root triggers (seconds).
pub(super) const QUICKSAND_TIME_THRESHOLD: f32 = 2.0;
/// Quicksand: root duration (seconds).
pub(super) const QUICKSAND_ROOT_DURATION: f32 = 1.5;

// --- Tier 3 talent constants ---

/// Death Garden: growth fraction over full duration.
pub(super) const DEATH_GARDEN_GROWTH_FRACTION: f32 = 0.3;
/// Death Garden: duration extension per kill (seconds).
pub(super) const DEATH_GARDEN_KILL_EXTENSION: f32 = 3.0;
/// Death Garden: maximum extra duration from kills (seconds).
pub(super) const DEATH_GARDEN_MAX_EXTENSION: f32 = 9.0;

/// Nature's Minefield: radius multiplier for each sub-zone.
pub(super) const MINEFIELD_RADIUS_MULT: f32 = 0.55;
/// Nature's Minefield: spread distance between sub-zone centers (fraction of base radius).
pub(super) const MINEFIELD_SPREAD_FRACTION: f32 = 0.7;

/// Spike Storm: interval between projectile volleys (seconds).
pub(super) const SPIKE_STORM_INTERVAL: f32 = 2.0;
/// Spike Storm: max targets per volley.
pub(super) const SPIKE_STORM_MAX_TARGETS: usize = 3;
/// Spike Storm: damage per projectile.
pub(super) const SPIKE_STORM_DAMAGE: f32 = 5.0;
/// Spike Storm: projectile speed.
pub(super) const SPIKE_STORM_PROJECTILE_SPEED: f32 = 300.0;
/// Spike Storm: targeting range multiplier (relative to zone radius).
pub(super) const SPIKE_STORM_RANGE_MULT: f32 = 1.5;
/// Spike Storm: projectile collision radius.
pub(super) const SPIKE_STORM_PROJECTILE_RADIUS: f32 = 5.0;
/// Unit collision radius for spike storm projectile hit detection.
pub(super) const UNIT_COLLISION_RADIUS: f32 = 8.0;
/// Spike Storm: projectile visual scale.
pub(super) const SPIKE_STORM_PROJECTILE_SCALE: f32 = 8.0;
