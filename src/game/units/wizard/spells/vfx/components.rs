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

/// Bright spark particle emitted on explosions and cast flares.
/// Independent entity that self-despawns.
#[derive(Component)]
pub struct FireSpark {
    /// World-space velocity.
    pub velocity: Vec3,
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base size of this spark.
    pub base_size: f32,
    /// Gravity strength (units/sec² applied over time).
    pub gravity: f32,
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

/// Poison cloud puff that drifts and swirls within a plague wind cloud.
/// Billboard particle that faces the camera, grows, then fades out.
#[derive(Component)]
pub struct PlagueSmoke {
    /// World-space velocity (gentle upward drift + lateral swirl).
    pub velocity: Vec3,
    /// Seconds since this puff was spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn (seconds).
    pub lifetime: f32,
    /// Base size of this smoke particle.
    pub base_size: f32,
    /// Phase offset for swirling motion.
    pub phase: f32,
    /// Y position at spawn time. Fire puffs use this for height-relative scaling/sway.
    /// Non-fire smoke (plague, fog) sets this to 0.0 to skip height effects.
    pub spawn_y: f32,
}

/// Tiny bright ember particle that flickers at the base of fire effects.
#[derive(Component)]
pub struct FireEmber {
    /// World-space velocity.
    pub velocity: Vec3,
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base size of this ember particle.
    pub base_size: f32,
    /// Random phase for flicker animation.
    pub phase: f32,
}

/// Marker on orange fire smoke puffs. Spawns a black smoke puff at apex growth.
#[derive(Component)]
pub struct FireOrangeSmokePuff {
    /// Whether the black smoke puff has already been emitted.
    pub emitted: bool,
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

/// Brief expanding glow + sparks at the wizard's staff when a spell is cast.
/// Color varies by spell school.
#[derive(Component)]
pub struct CastFlare {
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base size of the expanding glow ring.
    pub base_size: f32,
}

/// Floating mote particle for ambient spell zone effects (sleep, healing, etc.).
/// Drifts upward with gentle sway, fading over its lifetime.
#[derive(Component)]
pub struct FloatingMote {
    /// World-space velocity (primarily upward with lateral drift).
    pub velocity: Vec3,
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base size of this mote particle.
    pub base_size: f32,
    /// Random phase offset for sway animation.
    pub phase: f32,
}

/// Smoke poof particle for transformation effects (polymorph, banishment).
/// Expands quickly then fades.
#[derive(Component)]
pub struct SmokePoof {
    /// World-space velocity.
    pub velocity: Vec3,
    /// Seconds since spawned.
    pub time_alive: f32,
    /// Total lifetime before despawn.
    pub lifetime: f32,
    /// Base size of this poof particle.
    pub base_size: f32,
}
