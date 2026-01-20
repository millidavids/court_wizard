use bevy::prelude::*;

/// Base component for all spell projectiles.
///
/// Represents a spell projectile traveling through the battlefield.
#[derive(Component)]
pub struct Projectile {
    /// Direction the projectile is traveling (normalized vector).
    pub direction: Vec3,
    /// Speed of the projectile in units per second.
    pub speed: f32,
    /// Damage dealt on hit.
    pub damage: f32,
    /// Radius of the projectile for collision detection.
    pub radius: f32,
}

/// Marker component for spell effects (explosions, area effects, etc.).
#[derive(Component)]
pub struct SpellEffect {
    /// Time remaining before the effect despawns (in seconds).
    pub lifetime: f32,
}
