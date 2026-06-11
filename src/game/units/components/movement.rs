use bevy::prelude::*;

use crate::game::units::damage::DamageType;

/// Movement speed component for all units.
///
/// Determines how fast a unit moves in units per second.
#[derive(Component, Clone, Copy)]
pub struct MovementSpeed(pub f32);

/// Movement speed modifier from Commander aura as a percentage.
///
/// Applied to units within a Commander's aura range.
/// Examples: 0.25 = +25% speed from commander aura.
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct CommanderAuraSpeedModifier(pub f32);

/// Movement speed modifier from rough terrain as a percentage.
///
/// Applied to units walking over corpses.
/// Examples: -0.6 = -60% speed (0.4x multiplier).
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct RoughTerrainModifier(pub f32);

/// Unified movement speed slow modifier.
///
/// Replaces the separate FrostSlowModifier, SpikeGrowthSlowModifier, and
/// GreaseSlipModifier. Uses strongest-wins semantics: when a new slow is
/// applied, the stronger modifier and longer duration are kept.
#[derive(Component)]
pub struct SlowMovementModifier {
    /// Speed reduction as a percentage (negative value, e.g., -0.4 = 40% slower).
    pub modifier: f32,
    /// Time remaining before the slow effect expires (in seconds).
    pub time_remaining: f32,
}

impl SlowMovementModifier {
    /// Creates a new slow modifier with the given strength and duration.
    pub const fn new(modifier: f32, duration: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    /// Apply a new slow. Keeps the stronger modifier and longer duration.
    pub fn apply(&mut self, modifier: f32, duration: f32) {
        if modifier < self.modifier {
            self.modifier = modifier;
        }
        if duration > self.time_remaining {
            self.time_remaining = duration;
        }
    }
}

/// Progressive frost accumulation that drives blue tint, movement slow, and eventual freeze.
///
/// Each frost hit increases `level`. As level rises, the unit turns bluer and moves
/// slower. At level 1.0 the unit freezes solid (can't move or attack).
/// Frost decays after a delay when no new hits land.
#[derive(Component)]
pub struct FrostAccumulation {
    /// Current frost level (0.0 = none, 1.0 = frozen).
    pub level: f32,
    /// Seconds remaining before frost starts decaying (resets on each hit).
    pub decay_delay: f32,
}

impl FrostAccumulation {
    pub fn new(initial_level: f32, decay_delay: f32) -> Self {
        Self {
            level: initial_level.min(1.0),
            decay_delay,
        }
    }

    /// Add frost from a hit. Resets the decay delay.
    pub fn add_frost(&mut self, amount: f32, decay_delay: f32) {
        self.level = (self.level + amount).min(1.0);
        self.decay_delay = decay_delay;
    }
}

/// Targeting velocity toward target, set by the targeting system.
///
/// The targeting system calculates this based on the nearest enemy.
/// This is a normalized direction vector with distance information for weighting.
#[derive(Component, Default)]
pub struct TargetingVelocity {
    pub velocity: Vec3,
    pub distance_to_target: f32,
}

/// Per-unit multipliers for flocking forces.
///
/// Units without this component default to 1.0 for all forces.
/// Set individual fields to 0.0 to disable that force for a unit.
#[derive(Component)]
pub struct FlockingModifier {
    pub separation: f32,
    pub alignment: f32,
    pub cohesion: f32,
}

impl FlockingModifier {
    pub const fn new(separation: f32, alignment: f32, cohesion: f32) -> Self {
        Self {
            separation,
            alignment,
            cohesion,
        }
    }
}

/// Flocking velocity from separation, alignment, and cohesion forces.
///
/// The flocking system calculates this based on nearby allies.
/// This is a normalized direction vector.
#[derive(Component, Default)]
pub struct FlockingVelocity {
    pub velocity: Vec3,
    /// Direct repulsion force from smelly units, applied as raw acceleration
    /// (bypasses the weighted flocking normalization).
    pub smelly_repulsion: Vec3,
}

/// Knockback effect that moves a unit outward over time with decay.
/// Applied by ogre melee attacks, hag leaps, and meteor aftershock.
/// Decays linearly for a "tumbling through dirt" feel.
#[derive(Component)]
pub struct Knockback {
    /// Direction of knockback (normalized XZ).
    pub direction_x: f32,
    pub direction_z: f32,
    /// Initial knockback speed (units/s at full strength).
    pub speed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Time remaining before the effect expires.
    pub remaining: f32,
}

impl Knockback {
    pub fn new(direction: Vec3, speed: f32, duration: f32) -> Self {
        let normalized = direction.normalize_or_zero();
        Self {
            direction_x: normalized.x,
            direction_z: normalized.z,
            speed,
            duration,
            remaining: duration,
        }
    }
}

/// Airborne state for units launched into the air (geyser, explosions, etc.).
/// Applies gravity, offsets Y visually, and deals velocity-based fall damage on landing.
/// The unit is rooted separately via `RootedModifier` during flight.
#[derive(Component)]
pub struct Airborne {
    /// Current vertical velocity (positive = upward).
    pub vertical_velocity: f32,
    /// Current vertical offset from ground.
    pub height: f32,
    /// The unit's original Y position before launch (restored on landing).
    pub base_y: f32,
    /// Gravity acceleration applied per second (units/s²).
    pub gravity: f32,
    /// Damage type to apply on landing.
    pub damage_type: DamageType,
}

impl Airborne {
    /// Creates a new airborne state with the given launch velocity, gravity, and base Y.
    pub fn new(launch_velocity: f32, gravity: f32, base_y: f32, damage_type: DamageType) -> Self {
        Self {
            vertical_velocity: launch_velocity,
            height: 0.0,
            base_y,
            gravity,
            damage_type,
        }
    }
}

/// Damage scale factor for fall damage: `damage = abs(impact_velocity) * FALL_DAMAGE_SCALE`.
/// Calibrated so a geyser launch (200 velocity, 120 gravity → ~200 impact velocity) deals 15 damage.
pub const FALL_DAMAGE_SCALE: f32 = 0.075;

/// Component that slows units walking over rough terrain (corpses).
///
/// Applied to corpses to create a movement penalty for living units that walk over them.
#[derive(Component)]
pub struct RoughTerrain {
    /// Movement speed multiplier (0.0 = no movement, 1.0 = full speed).
    /// For example, 0.6 means units move at 60% of their normal speed.
    pub slowdown_factor: f32,
}
