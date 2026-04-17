use bevy::prelude::*;

/// Pond color (deep blue, sampled from wall_floor.png water pool center).
pub const POND_COLOR: Color = Color::srgba(0.13, 0.35, 0.65, 0.7);

/// Minimum radius for a pond (world units).
pub const POND_RADIUS_MIN: f32 = 80.0;

/// Maximum radius for a pond (world units).
pub const POND_RADIUS_MAX: f32 = 120.0;

/// Flow field cost for pond cells.
pub const POND_FLOW_COST: f32 = 2.5;

/// Electric damage multiplier for wet units (50% more electric arc damage).
pub const WET_ELECTRIC_DAMAGE_MULTIPLIER: f32 = 1.5;

/// Base number of ponds per level (before terrain density scaling).
pub const POND_BASE_COUNT_MIN: u32 = 1;
pub const POND_BASE_COUNT_MAX: u32 = 4;

/// Y position for pond surface (slightly above ground).
pub const POND_SURFACE_Y: f32 = 2.0;

/// Interval between ripple spawns per pond (seconds).
pub const POND_RIPPLE_INTERVAL: f32 = 1.5;

/// How long each pond ripple lives (seconds).
pub const POND_RIPPLE_LIFETIME: f32 = 3.0;

/// Peak alpha for pond ripples.
pub const POND_RIPPLE_ALPHA: f32 = 0.35;

// ── Fire evaporation → fog cloud ────────────────────────────────────────

/// How long (seconds) the fog cloud lingers after the last fire contribution before it dissipates.
pub const FOG_LINGER_DURATION: f32 = 10.0;

/// Fog intensity added per unit of fire damage. One fireball (~50 total damage) adds ~1.0
/// intensity → cloud radius equals the pond. Sustained fire grows it past pond size.
pub const FOG_PER_DAMAGE: f32 = 0.02;

/// Maximum fog intensity considered for cloud radius scaling. At 3.0 the cloud reaches 3×
/// pond radius — a large visible cap for repeated fire damage.
pub const FOG_INTENSITY_MAX: f32 = 3.0;

/// Interval between fog-puff spawns per pond (seconds). Matches the fog-cloud spell cadence.
pub const POND_FOG_SPAWN_INTERVAL: f32 = 0.08;

/// Base number of fog puffs spawned per pond per interval (scales up with intensity).
pub const POND_FOG_COUNT_PER_SPAWN: usize = 4;

// ── Frost → frozen pond ──────────────────────────────────────────────────

/// Pond color when fully frozen (lerp target). Lighter/whiter than the liquid blue.
pub const FROZEN_POND_COLOR: Color = Color::srgba(0.6, 0.8, 1.0, 0.7);

/// Pathfinding cost for frozen ponds. Higher than liquid ponds (2.5) so units route around more.
pub const POND_FROZEN_FLOW_COST: f32 = 5.0;

/// Freeze level added per unit of frost damage. A single Squall ice explosion (~10 damage) adds 0.1.
pub const POND_FREEZE_PER_DAMAGE: f32 = 0.01;

/// Seconds of no frost contribution before the pond starts thawing.
pub const POND_FREEZE_DECAY_DELAY: f32 = 3.0;

/// Thaw rate (freeze-level per second) after the decay delay elapses.
pub const POND_FREEZE_DECAY_RATE: f32 = 0.1;

/// Freeze-level threshold at which the pond flips to the frozen pathfinding cost and
/// applies slowdown to units walking over it.
pub const POND_FREEZE_PATHFINDING_THRESHOLD: f32 = 0.5;

/// Speed multiplier applied to units crossing a fully frozen pond (-0.5 = 50% slower).
pub const FROZEN_POND_SPEED_MODIFIER: f32 = -0.5;

// ── Electric → shocked pond ──────────────────────────────────────────────

/// Seconds the pond remains shocked after the last electric contribution.
pub const POND_SHOCK_DURATION: f32 = 6.0;

/// Cooldown between arc pulses from a shocked pond.
pub const POND_SHOCK_ARC_COOLDOWN: f32 = 0.8;

/// Arc range from a shocked pond (center) to nearby units. Larger than unit `ELECTRIC_ARC_RANGE` (80.0).
pub const POND_SHOCK_ARC_RADIUS: f32 = 200.0;

/// Damage dealt per arc pulse to each hit unit.
pub const POND_SHOCK_ARC_DAMAGE: f32 = 8.0;

/// Maximum units arced to per pulse.
pub const POND_SHOCK_MAX_TARGETS: usize = 4;
