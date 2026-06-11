use bevy::prelude::*;

use crate::game::units::DamageType;

/// Fireball projectile component.
///
/// Represents a fireball traveling through the battlefield until it hits a target or the ground.
#[derive(Component)]
pub struct Fireball {
    /// Velocity vector of the fireball.
    pub velocity: Vec3,
    /// Damage dealt by the explosion.
    pub damage: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Radius of the explosion when fireball impacts.
    pub explosion_radius: f32,
    /// Collision radius of the projectile itself.
    pub radius: f32,
    /// Whether this fireball is empowered (for residual effect scaling).
    pub empowerment: f32,
    /// Cluster Bomb talent: on impact, spawn 3 mini-fireballs.
    pub cluster_bomb: bool,
    /// Napalm talent: leaves burning trail while flying.
    pub napalm: bool,
    /// Napalm timer for periodic trail spawning.
    pub napalm_timer: f32,
    /// Scorched Earth talent: explosion leaves persistent burning ground.
    pub scorched_earth: bool,
    /// Chain Ignition talent: hit enemies take +50% damage for 3s.
    pub chain_ignition: bool,
    /// Override explosion duration (0.0 = use default).
    pub explosion_duration: f32,
}

impl Fireball {
    /// Creates a new Fireball component.
    pub const fn new(
        velocity: Vec3,
        damage: f32,
        damage_type: DamageType,
        explosion_radius: f32,
        radius: f32,
        empowerment: f32,
    ) -> Self {
        Self {
            velocity,
            damage,
            damage_type,
            explosion_radius,
            radius,
            empowerment,
            cluster_bomb: false,
            napalm: false,
            napalm_timer: 0.0,
            scorched_earth: false,
            chain_ignition: false,
            explosion_duration: 0.0,
        }
    }
}

/// Marker for Scorched Earth persistent fire zones (enables fire smoke VFX).
#[derive(Component)]
pub struct ScorchedEarthFire;

/// Pre-generated sub-explosion that triggers when the main explosion reaches its distance.
pub(crate) struct PendingBubble {
    /// Normalized direction from explosion center.
    pub direction: Vec3,
    /// Distance from center at which this bubble triggers.
    pub distance: f32,
    /// Max radius of the sub-explosion.
    pub radius: f32,
}

/// Spawner for sub-explosion bubbles that break up the main explosion's silhouette.
///
/// Attached only to fireball explosions using the sphere mesh. Each pending bubble
/// triggers when the main explosion's growing radius reaches its offset distance,
/// giving an amorphous, bubbling eruption look.
#[derive(Component)]
pub(crate) struct ExplosionBubbleSpawner {
    pub pending: Vec<PendingBubble>,
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
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time the explosion has been active (in seconds).
    pub time_alive: f32,
    /// Time since last damage tick (in seconds).
    pub time_since_last_tick: f32,
    /// Empowerment multiplier for spell effectiveness.
    #[allow(dead_code)]
    pub empowerment: f32,
    /// Duration of this explosion (allows per-explosion override, e.g. Lingering Flames).
    pub duration: f32,
    /// Chain Ignition talent: apply damage amplification debuff to hit enemies.
    pub chain_ignition: bool,
    /// Skip radius growth — start at full size immediately.
    pub skip_growth: bool,
    /// Which spell created this explosion (for talent progress tracking).
    pub source_spell: crate::game::units::wizard::components::Spell,
    /// Whether VFX (sparks + smoke) have been spawned for this explosion.
    pub vfx_spawned: bool,
    /// Growth speed multiplier (1.0 = linear over full duration, higher = reach max sooner).
    pub growth_speed: f32,
}

impl FireballExplosion {
    /// Creates a new FireballExplosion component.
    pub fn new(
        origin: Vec3,
        max_radius: f32,
        damage_per_tick: f32,
        damage_type: DamageType,
        empowerment: f32,
    ) -> Self {
        use super::constants;
        Self {
            origin,
            max_radius,
            damage_per_tick,
            damage_type,
            time_alive: 0.0,
            time_since_last_tick: 0.0,
            empowerment,
            duration: constants::EXPLOSION_DURATION,
            chain_ignition: false,
            skip_growth: false,
            source_spell: crate::game::units::wizard::components::Spell::Fireball,
            vfx_spawned: false,
            growth_speed: 1.0,
        }
    }

    /// Returns the current radius of the explosion based on how long it's been active.
    pub fn current_radius(&self) -> f32 {
        if self.skip_growth || self.duration <= 0.0 {
            return self.max_radius;
        }

        let growth_factor = (self.time_alive / self.duration * self.growth_speed).min(1.0);
        self.max_radius * growth_factor
    }
}
