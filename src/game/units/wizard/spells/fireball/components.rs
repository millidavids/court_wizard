use bevy::prelude::*;

/// Fireball projectile component.
///
/// Represents a fireball traveling through the battlefield until it hits a target or the ground.
#[derive(Component)]
pub struct Fireball {
    /// Velocity vector of the fireball.
    pub velocity: Vec3,
    /// Damage dealt by the explosion.
    pub damage: f32,
    /// Radius of the explosion when fireball impacts.
    pub explosion_radius: f32,
    /// Collision radius of the projectile itself.
    pub radius: f32,
}

impl Fireball {
    /// Creates a new Fireball component.
    pub const fn new(velocity: Vec3, damage: f32, explosion_radius: f32, radius: f32) -> Self {
        Self {
            velocity,
            damage,
            explosion_radius,
            radius,
        }
    }
}

/// Fireball explosion component.
///
/// Represents the expanding sphere explosion after a fireball impacts.
#[derive(Component)]
pub struct FireballExplosion {
    /// Center point of the explosion.
    pub origin: Vec3,
    /// Maximum radius the explosion will reach.
    pub max_radius: f32,
    /// Damage dealt per tick to units hit by the explosion.
    pub damage_per_tick: f32,
    /// Time the explosion has been active (in seconds).
    pub time_alive: f32,
    /// Time since last damage tick (in seconds).
    pub time_since_last_tick: f32,
}

impl FireballExplosion {
    /// Creates a new FireballExplosion component.
    pub fn new(origin: Vec3, max_radius: f32, damage_per_tick: f32) -> Self {
        Self {
            origin,
            max_radius,
            damage_per_tick,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
        }
    }

    /// Returns the current radius of the explosion based on how long it's been active.
    pub fn current_radius(&self, duration: f32) -> f32 {
        if duration <= 0.0 {
            return self.max_radius;
        }

        let growth_factor = (self.time_alive / duration).min(1.0);
        self.max_radius * growth_factor
    }
}
