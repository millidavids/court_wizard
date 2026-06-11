use super::colors::UNIT_SCALE;

// ===== Unit Stats =====

/// Default health for all units.
pub const UNIT_HEALTH: f32 = 100.0;

/// Default movement speed for all units (units per second).
pub const UNIT_MOVEMENT_SPEED: f32 = 172.5;

/// Global multiplier applied to all unit movement speeds.
/// 1.0 = normal speed, 0.8 = 20% slower, etc.
pub const GLOBAL_SPEED_MULTIPLIER: f32 = 0.55;

/// Hitbox height for defender units.
pub const DEFENDER_HITBOX_HEIGHT: f32 = 25.0 * UNIT_SCALE;

/// Hitbox height for attacker units.
pub const ATTACKER_HITBOX_HEIGHT: f32 = 20.0 * UNIT_SCALE;

// ===== Movement Constants =====

/// Velocity damping coefficient (reduces velocity each frame to prevent excessive momentum).
pub const VELOCITY_DAMPING: f32 = 0.85;

/// Steering force strength for acceleration-based movement.
pub const STEERING_FORCE: f32 = 500.0;

/// Movement speed multiplier when in melee combat (slows units down to prevent running around).
pub const MELEE_SLOWDOWN_FACTOR: f32 = 0.3;

/// Distance threshold to be considered "in melee" for slowdown purposes.
pub const MELEE_SLOWDOWN_DISTANCE: f32 = 50.0 * UNIT_SCALE;

// ===== Flocking Constants =====

/// Maximum distance to consider a unit as a neighbor for flocking behavior.
pub const NEIGHBOR_DISTANCE: f32 = 100.0;

/// Distance threshold for separation force to apply.
/// Units only separate when they're very close to colliding (just beyond hitbox radius).
pub const SEPARATION_DISTANCE: f32 = 3.0 * UNIT_SCALE;

/// Strength of the separation force (pushes units apart).
/// Since we're using normalized directions, this should be small (0-1 range).
pub const SEPARATION_STRENGTH: f32 = 0.05;

/// Strength of the alignment force (matches neighbor velocities).
/// Since we're using normalized directions, this should be small (0-1 range).
pub const ALIGNMENT_STRENGTH: f32 = 0.1;

/// Strength of the cohesion force (pulls units toward group center). Set to 0.0 to disable.
pub const COHESION_STRENGTH: f32 = 0.1;

/// Maximum allowed overlap between hitboxes as a percentage. 0.0 = no overlap allowed.
pub const MAX_OVERLAP_PERCENT: f32 = 0.2;

/// Minimum distance threshold for collision calculations (avoids division by zero).
pub const MIN_DISTANCE_THRESHOLD: f32 = 0.01;

/// Collision resolution iterations (higher = more accurate but more expensive).
pub const COLLISION_ITERATIONS: u32 = 4;

// ===== Combat Constants =====

/// Attack range multiplier based on hitbox radius.
pub const ATTACK_RANGE_MULTIPLIER: f32 = 1.5;

/// Damage dealt per attack.
pub const ATTACK_DAMAGE: f32 = 10.0;

/// Duration of one complete attack cycle in seconds.
pub const ATTACK_CYCLE_DURATION: f32 = 2.0;

// ===== Effectiveness System =====

/// Bonus to effectiveness per ally in melee range (+10% each).
pub const EFFECTIVENESS_ALLY_BONUS_PER_UNIT: f32 = 0.10;

/// Penalty to effectiveness per enemy in melee range (-15% each).
pub const EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT: f32 = -0.15;

/// Minimum effectiveness cap (10% of base).
pub const EFFECTIVENESS_MIN: f32 = 0.1;

/// Maximum effectiveness cap (200% of base).
pub const EFFECTIVENESS_MAX: f32 = 2.0;

/// Flat effectiveness boost applied to attackers while a co-op guest is helping
/// the host defend (endless/roguelite co-op only). Removed the instant the guest
/// disconnects.
pub const COOP_ATTACKER_EFFECTIVENESS_BONUS: f32 = 0.30;

// ===== Gameplay Constants =====

/// Initial number of defenders spawned at game start.
pub const INITIAL_DEFENDER_COUNT: u32 = 100;

/// Number of King's Guard units spawned at game start.
pub const KINGS_GUARD_COUNT: u32 = 10;

/// Radius of the orbit circle for King's Guard around the King.
pub const KINGS_GUARD_ORBIT_RADIUS: f32 = 20.0 * UNIT_SCALE;

// ===== King's Retreat =====

/// Percentage of the wizard's spell range at which the King triggers a retreat.
/// At 90% of spell range, the King is dangerously close to the edge.
pub const RETREAT_TRIGGER_DISTANCE_PERCENT: f32 = 0.90;

/// Duration of retreat in seconds — defenders disengage and fall back to spawn.
pub const RETREAT_DURATION_SECS: f32 = 15.0;

// ===== Defender Activation =====

/// Distance at which defenders activate (XZ plane distance).
/// Once any enemy is within this range of any defender, all defenders activate.
pub const DEFENDER_ACTIVATION_RANGE: f32 = 800.0;
