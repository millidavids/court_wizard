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

/// Marks a flame particle that is a visual-only ghost of the opponent's
/// flamethrower (spawned from a replicated cast event). Excluded from
/// `check_flame_collisions` (no double damage — the real damage already crosses
/// via CRDT) and from the ground-fire spawn in `despawn_expired_flames` (the
/// burning patch arrives separately as a networked spell effect).
#[derive(Component)]
pub struct GhostFlameParticle;

/// Marks a muzzle flash / bullet tracer that is a visual-only ghost of the
/// opponent's gunfire. Excluded from the `replicate_gun_*` emitters so a
/// received ghost is never re-shipped back to the sender (which would otherwise
/// ping-pong forever in a Warglock-vs-Warglock match).
#[derive(Component)]
pub struct GhostMuzzleFlash;

/// See [`GhostMuzzleFlash`].
#[derive(Component)]
pub struct GhostBulletTracer;

/// Muzzle flash visual effect.
#[derive(Component)]
pub struct MuzzleFlash {
    pub timer: f32,
    /// The gun that fired this flash. Read by `replicate_gun_muzzle_flashes` to
    /// play the correct shot sound on the opponent even if the player has since
    /// switched guns — reading the live `GunState` would race a rapid switch and
    /// ship the wrong gunshot.
    pub gun: GunType,
}
