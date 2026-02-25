use bevy::prelude::*;

/// Straight-line projectile that flies toward the ground.
/// Detonates on battlefield impact (y<=0) or lifetime expiry, spawning a DispelImpact.
#[derive(Component)]
pub struct DispelProjectile {
    /// Velocity vector (direction * speed).
    pub velocity: Vec3,
    /// Remaining lifetime before forced despawn.
    pub lifetime: f32,
}

/// Cooldown timer for wizard dispel casting.
#[derive(Component)]
pub struct DispelCooldown {
    pub remaining: f32,
}

/// Expanding translucent sphere that dispels spell effects it overlaps.
#[derive(Component)]
pub struct DispelImpact {
    /// Time this impact has been alive (seconds).
    pub time_alive: f32,
    /// Total duration before despawn (seconds).
    pub duration: f32,
}
