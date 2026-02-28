//! Shared visual effect components.

use bevy::prelude::*;

/// Glow halo that follows a fire projectile entity.
#[derive(Component)]
pub struct FireGlow {
    /// The projectile entity this glow tracks.
    pub source_entity: Entity,
    /// Base visual radius of the source projectile (used to scale glow).
    pub base_radius: f32,
}

/// Smoke wisp that drifts upward and self-dissipates.
/// Independent entity — persists after the source is despawned.
#[derive(Component)]
pub struct FireSmoke {
    /// World-space velocity (primarily upward with lateral spread).
    pub velocity: Vec3,
    /// Seconds since this wisp was spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn (seconds).
    pub lifetime: f32,
    /// Base size of this smoke particle.
    pub base_size: f32,
}

/// Bright spark particle emitted on fire explosions.
/// Independent entity that self-despawns.
#[derive(Component)]
pub struct FireSpark {
    /// World-space velocity.
    pub velocity: Vec3,
    /// Seconds since spawned.
    pub time_alive: f32,
}

/// Glow halo that follows a magic missile projectile.
#[derive(Component)]
pub struct MissileGlow {
    /// The missile entity this glow tracks.
    pub source_entity: Entity,
    /// Base visual radius (used to scale glow).
    pub base_radius: f32,
}

/// Semi-transparent dithered billboard that sways and bobs near hot objects,
/// creating a lo-fi heat haze effect.
#[derive(Component)]
pub struct HeatShimmer {
    /// Gentle lateral sway velocity.
    pub velocity: Vec3,
    /// Seconds since this shimmer was spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn (seconds).
    pub lifetime: f32,
    /// Base size of this shimmer particle.
    pub base_size: f32,
    /// Random phase offset for dither animation.
    pub phase: f32,
}

/// White sparkle particle that trails behind a magic missile.
/// Spawns at the missile's position, inherits its velocity, then decelerates
/// to create a comet-like trail effect.
#[derive(Component)]
pub struct MissileSparkle {
    /// World-space velocity (decelerates over time).
    pub velocity: Vec3,
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base size of this sparkle particle.
    pub base_size: f32,
}
