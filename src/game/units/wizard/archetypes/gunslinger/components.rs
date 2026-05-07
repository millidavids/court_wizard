use bevy::prelude::*;

use super::constants;

/// The five gun types available to the gunslinger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GunType {
    MachineGun,
    Magnum,
    RocketLauncher,
    Shotgun,
    Flamethrower,
}

impl GunType {
    /// Returns all gun types in slot order (1-5).
    pub const fn all() -> [GunType; 5] {
        [
            GunType::MachineGun,
            GunType::Magnum,
            GunType::Shotgun,
            GunType::RocketLauncher,
            GunType::Flamethrower,
        ]
    }

    /// Returns max ammo for this gun.
    pub const fn max_ammo(&self) -> u32 {
        match self {
            GunType::MachineGun => constants::MACHINE_GUN_MAX_AMMO,
            GunType::Magnum => constants::MAGNUM_MAX_AMMO,
            GunType::RocketLauncher => constants::ROCKET_MAX_AMMO,
            GunType::Shotgun => constants::SHOTGUN_MAX_AMMO,
            GunType::Flamethrower => constants::FLAMETHROWER_MAX_AMMO,
        }
    }

    /// Returns reload time in seconds.
    pub const fn reload_time(&self) -> f32 {
        match self {
            GunType::MachineGun => constants::MACHINE_GUN_RELOAD_TIME,
            GunType::Magnum => constants::MAGNUM_RELOAD_TIME,
            GunType::RocketLauncher => constants::ROCKET_RELOAD_TIME,
            GunType::Shotgun => constants::SHOTGUN_RELOAD_TIME,
            GunType::Flamethrower => constants::FLAMETHROWER_RELOAD_TIME,
        }
    }

    /// Returns minimum time between shots.
    #[allow(dead_code)]
    pub const fn fire_interval(&self) -> f32 {
        match self {
            GunType::MachineGun => constants::MACHINE_GUN_FIRE_INTERVAL,
            GunType::Magnum => constants::MAGNUM_FIRE_INTERVAL,
            GunType::RocketLauncher => constants::ROCKET_FIRE_INTERVAL,
            GunType::Shotgun => constants::SHOTGUN_FIRE_INTERVAL,
            GunType::Flamethrower => constants::FLAMETHROWER_FIRE_INTERVAL,
        }
    }

    /// Returns true if this gun fires while mouse is held (vs click-to-fire).
    #[allow(dead_code)]
    pub const fn is_hold_to_fire(&self) -> bool {
        matches!(self, GunType::MachineGun | GunType::Flamethrower)
    }

    /// Returns how many ammo pieces to show in the UI per actual ammo unit.
    pub const fn ammo_per_ui_piece(&self) -> u32 {
        match self {
            GunType::MachineGun => 5,    // 60/5 = 12 pieces
            GunType::Flamethrower => 10, // 100/10 = 10 pieces
            _ => 1,
        }
    }
}

/// Visual-only bullet tracer — flies fast, no damage (damage is applied instantly via hitscan).
#[derive(Component)]
pub struct BulletTracer {
    pub velocity: Vec3,
    pub max_range: f32,
    pub origin: Vec3,
}

/// Invisible hitscan ray spawned on fire. Checked for collisions once, then despawned.
#[derive(Component)]
pub struct HitscanRay {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_range: f32,
    pub cylinder_radius: f32,
    pub damage: f32,
}

/// Flame particle component.
#[derive(Component)]
pub struct FlameParticle {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: f32,
    pub time_alive: f32,
    pub radius: f32,
}

/// Muzzle flash visual effect.
#[derive(Component)]
pub struct MuzzleFlash {
    pub timer: f32,
}

