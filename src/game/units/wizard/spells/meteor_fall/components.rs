//! Meteor Fall spell components.

use bevy::prelude::*;

/// Meteor Fall storm component - invisible marker entity that spawns meteor projectiles.
///
/// The storm persists as long as the wizard maintains concentration.
/// Periodically spawns meteor projectiles that rain down within the storm radius.
#[derive(Component)]
pub(crate) struct MeteorFallStorm {
    /// Center position of the storm in world space.
    pub position: Vec3,
    /// Radius of the storm area.
    pub radius: f32,
    /// Time since the storm was created (for animations/effects).
    pub time_alive: f32,
    /// Time since last meteor projectile spawn.
    pub time_since_spawn: f32,
    /// Empowerment multiplier for spell effectiveness.
    pub empowerment: f32,
    // --- Talent fields ---
    /// Pre-computed spawn interval (from talents).
    pub spawn_interval: f32,
    /// Damage multiplier from talents (Scorching Impact).
    pub damage_mult: f32,
    /// Explosion radius multiplier from talents.
    pub explosion_radius_mult: f32,
    /// Ground fire duration multiplier from talents.
    pub ground_fire_duration_mult: f32,
    /// Ground fire damage multiplier from talents.
    pub ground_fire_damage_mult: f32,
    /// Ground fire radius multiplier from talents.
    pub ground_fire_radius_mult: f32,
    /// Whether meteors track enemies.
    pub tracking: bool,
    /// Whether meteors apply aftershock knockback.
    pub aftershock: bool,
    /// Whether Extinction Event is enabled.
    pub extinction_event: bool,
    /// Whether the extinction meteor has already fired.
    pub extinction_fired: bool,
    /// Whether Volcanic Eruption is enabled.
    pub volcanic_eruption: bool,
    /// Visual mesh radius multiplier from talents.
    pub mesh_radius_mult: f32,
    /// Optional fixed duration (seconds). When set, storm self-destructs after this time
    /// instead of requiring concentration. Used by Extinction Event.
    pub duration: Option<f32>,
}

impl MeteorFallStorm {
    /// Creates a new meteor fall storm at the specified position.
    pub fn new(position: Vec3, radius: f32, empowerment: f32) -> Self {
        Self {
            position,
            radius,
            time_alive: 0.0,
            time_since_spawn: 0.0,
            empowerment,
            spawn_interval: super::constants::METEOR_SPAWN_INTERVAL,
            damage_mult: 1.0,
            explosion_radius_mult: 1.0,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            tracking: false,
            aftershock: false,
            extinction_event: false,
            extinction_fired: false,
            volcanic_eruption: false,
            mesh_radius_mult: 1.0,
            duration: None,
        }
    }

    /// Updates timers.
    pub fn update_timers(&mut self, delta: f32) {
        self.time_alive += delta;
        self.time_since_spawn += delta;
    }

    /// Resets the spawn timer.
    pub fn reset_spawn_timer(&mut self) {
        self.time_since_spawn = 0.0;
    }
}

/// Meteor projectile component - falls from the sky and explodes on impact.
#[derive(Component)]
pub(crate) struct MeteorProjectile {
    /// Current velocity of the projectile.
    pub velocity: Vec3,
    /// Damage dealt by the explosion.
    pub damage: f32,
    /// Radius of the explosion.
    pub explosion_radius: f32,
    /// Empowerment multiplier.
    pub empowerment: f32,
    /// Visual mesh radius (for deferred glow spawning).
    pub mesh_radius: f32,
    /// Whether the glow sibling has been spawned yet.
    pub has_glow: bool,
    // --- Talent fields ---
    /// Whether this projectile applies aftershock knockback.
    pub aftershock: bool,
    /// Whether this projectile triggers volcanic eruption.
    pub volcanic_eruption: bool,
    /// Ground fire duration multiplier.
    pub ground_fire_duration_mult: f32,
    /// Ground fire damage multiplier.
    pub ground_fire_damage_mult: f32,
    /// Ground fire radius multiplier.
    pub ground_fire_radius_mult: f32,
    /// Whether this projectile tracks enemies.
    pub tracking: bool,
    /// Whether this is the Extinction Event meteor (triggers flow field recalc on ground fire).
    pub is_extinction: bool,
}

impl MeteorProjectile {
    /// Creates a new meteor projectile.
    pub const fn new(
        velocity: Vec3,
        damage: f32,
        explosion_radius: f32,
        empowerment: f32,
        mesh_radius: f32,
    ) -> Self {
        Self {
            velocity,
            damage,
            explosion_radius,
            empowerment,
            mesh_radius,
            has_glow: false,
            aftershock: false,
            volcanic_eruption: false,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            tracking: false,
            is_extinction: false,
        }
    }
}

/// Meteor explosion component - visual and damage effect on impact.
#[derive(Component)]
pub(crate) struct MeteorExplosion {
    /// Center point of the explosion.
    pub origin: Vec3,
    /// Maximum radius the explosion will reach.
    pub max_radius: f32,
    /// Damage dealt to units in the explosion.
    pub damage: f32,
    /// Time the explosion has been active (in seconds).
    pub time_alive: f32,
    /// Whether damage has been applied yet (one-time damage).
    pub damage_applied: bool,
}

impl MeteorExplosion {
    /// Creates a new meteor explosion.
    pub const fn new(origin: Vec3, max_radius: f32, damage: f32) -> Self {
        Self {
            origin,
            max_radius,
            damage,
            time_alive: 0.0,
            damage_applied: false,
        }
    }

    /// Returns the current radius of the explosion based on how long it's been active.
    pub fn current_radius(&self, growth_time: f32) -> f32 {
        if growth_time <= 0.0 {
            return self.max_radius;
        }

        let growth_factor = (self.time_alive / growth_time).min(1.0);
        self.max_radius * growth_factor
    }
}

/// Persistent burning ground left by meteor impacts.
///
/// Damages units walking through the fire zone and acts as a pathfinding hazard.
#[derive(Component)]
pub(crate) struct MeteorGroundFire {
    /// Center position of the fire zone.
    pub origin: Vec3,
    /// Radius of the fire zone.
    pub radius: f32,
    /// Damage dealt per tick to units inside.
    pub damage_per_tick: f32,
    /// Time between damage ticks (seconds).
    pub tick_interval: f32,
    /// Total duration the fire persists (seconds).
    pub duration: f32,
    /// Time since the fire was created.
    pub time_alive: f32,
    /// Time since last damage tick.
    pub time_since_last_tick: f32,
    /// Volcanic Eruption: count of additional meteor hits on this fire zone.
    pub eruption_charges: u32,
    /// Whether this is the Extinction Event ground fire (triggers flow field recalc).
    pub is_extinction: bool,
}

impl MeteorGroundFire {
    /// Creates a new ground fire zone.
    pub fn new(origin: Vec3, radius: f32, damage: f32, tick: f32, duration: f32) -> Self {
        Self {
            origin,
            radius,
            damage_per_tick: damage,
            tick_interval: tick,
            duration,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
            eruption_charges: 0,
            is_extinction: false,
        }
    }
}
