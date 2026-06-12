//! Spell effect snapshot types: persistent effects, projectiles, arcs.

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

/// Identifies the type of persistent spell effect for guest spawning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SpellEffectKind {
    // Zones (Circle meshes, face up)
    SpikeGrowthZone = 0,
    HealingPlumeZone = 1,
    EntangleGround = 2,
    FogCloudZone = 3,
    GreaseZone = 4,
    GreaseFire = 5,
    PlagueWindCloud = 6,
    MeteorGroundFire = 7,
    // Objects (Sphere meshes)
    BlackHole = 10,
    ArcaneCrystal = 11,
    LightningRod = 12,
    // Walls (Cuboid meshes)
    WallOfStone = 20,
    WallOfFire = 21,
    // Explosions (Sphere meshes, growing scale)
    FireballExplosion = 30,
    MeteorExplosion = 31,
    IceExplosion = 32,
    DispelImpact = 33,
    // Storm parents (invisible markers driving reticles / mist visuals).
    SquallStorm = 40,
    // Ground hazards
    ScorchedEarthFire = 50,
    NapalmTrail = 51,
    // Boulder lifecycle — Brute / Ogre throw boulders that fly in an arc
    // then become persistent pathfinding obstacles. Both phases ride on the
    // existing `SpellEffectSnapshot` (Transform-only) infrastructure; the
    // landed-boulder snapshot encodes `sprite_index` in `extra[0]` so the
    // guest picks the matching asset.
    BoulderProjectileEffect = 60,
    BoulderObstacle = 61,
    // Warglock flamethrower ground fire — the persistent burning patch left on
    // the ground. `extra[0]` = radius, `extra[1]` = remaining duration,
    // `extra[2]` = damage-per-tick, `extra[3]` = 1.0 if growth-suppressed.
    FlameGroundFire = 70,
}

impl TryFrom<u8> for SpellEffectKind {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SpikeGrowthZone),
            1 => Ok(Self::HealingPlumeZone),
            2 => Ok(Self::EntangleGround),
            3 => Ok(Self::FogCloudZone),
            4 => Ok(Self::GreaseZone),
            5 => Ok(Self::GreaseFire),
            6 => Ok(Self::PlagueWindCloud),
            7 => Ok(Self::MeteorGroundFire),
            10 => Ok(Self::BlackHole),
            11 => Ok(Self::ArcaneCrystal),
            12 => Ok(Self::LightningRod),
            20 => Ok(Self::WallOfStone),
            21 => Ok(Self::WallOfFire),
            30 => Ok(Self::FireballExplosion),
            31 => Ok(Self::MeteorExplosion),
            32 => Ok(Self::IceExplosion),
            33 => Ok(Self::DispelImpact),
            40 => Ok(Self::SquallStorm),
            50 => Ok(Self::ScorchedEarthFire),
            51 => Ok(Self::NapalmTrail),
            60 => Ok(Self::BoulderProjectileEffect),
            61 => Ok(Self::BoulderObstacle),
            70 => Ok(Self::FlameGroundFire),
            _ => Err(()),
        }
    }
}

/// Persistent spell effect snapshot (~44 bytes).
///
/// Sent every frame. The guest uses it at spawn time for initial parameters
/// and on subsequent frames only checks existence (for force-despawn).
#[derive(Serialize, Deserialize)]
pub struct SpellEffectSnapshot {
    /// Stable network entity ID for lifecycle tracking.
    pub net_id: u32,
    /// Spell effect type (SpellEffectKind as u8).
    pub kind: u8,
    /// World position X.
    pub x: f32,
    /// World position Y (height).
    pub y: f32,
    /// World position Z.
    pub z: f32,
    /// Y-axis rotation in radians (for walls).
    pub rotation_y: f32,
    /// Kind-specific initialization data (radius, duration, empowerment, etc.).
    pub extra: [f32; 4],
    /// Kind-specific talent bit flags. Each `SpellEffectKind`'s collector
    /// packs talent booleans into this u32 so the ghost reconstructs the
    /// host's talent-modified behaviour rather than running on defaults.
    /// Bit layouts are local to each spell — see the per-spell collector
    /// and `spawn_spell_effect` arm for the bit definitions.
    pub flags: u32,
}

/// Ephemeral spell projectile snapshot (~13 bytes).
///
/// Despawned and re-spawned each frame on the guest, like arrows.
#[derive(Serialize, Deserialize)]
pub struct SpellProjectileSnapshot {
    /// Projectile type: 0=Fireball, 1=IceProjectile, 2=MeteorProjectile,
    /// 3=DispelProjectile.
    pub kind: u8,
    /// World position X.
    pub x: f32,
    /// World position Y.
    pub y: f32,
    /// World position Z.
    pub z: f32,
    /// Sphere radius the local caster spawned the projectile at — needed so
    /// the receiver can spawn a visually identical ghost (talents and
    /// empowerment scale this from the caster's `PrimedSpell` /
    /// `Fireball::radius`, neither of which is otherwise networked).
    ///
    /// **Wire-format note:** `bincode` is positional and field-tagless, so
    /// adding/removing/reordering any field of this struct is a breaking
    /// change. MP requires same-version peers; consider promoting the
    /// protocol to a versioned/tagged encoding if cross-version sessions
    /// are ever supported.
    pub scale: f32,
}

/// Ephemeral spell arc/beam snapshot (~25 bytes).
///
/// Despawned and re-spawned each frame on the guest.
#[derive(Serialize, Deserialize)]
pub struct SpellArcSnapshot {
    /// Arc type: 0=ChainLightning, 1=LightningStrike,
    /// 4=FingerOfDeath, 5=LightningRodArc.
    /// Kinds 2 (crystal_beam) and 3 (crystal_arc) were valid historically but
    /// are no longer emitted — the receiver still handles them as dead paths.
    /// Kind 6 (Disintegrate) was retired; Disintegrate now uses `BeamSnapshot`.
    pub kind: u8,
    /// Origin X.
    pub ox: f32,
    /// Origin Y.
    pub oy: f32,
    /// Origin Z.
    pub oz: f32,
    /// Target/end X.
    pub tx: f32,
    /// Target/end Y.
    pub ty: f32,
    /// Target/end Z.
    pub tz: f32,
}

/// Resource holding pre-collected spell snapshot data.
///
/// Written by `collect_spell_snapshots` and consumed by `send_state_snapshots`.
/// Avoids exceeding Bevy's 16-parameter limit on a single system.
#[derive(Resource, Default)]
pub struct SpellSnapshotData {
    pub spell_effects: Vec<SpellEffectSnapshot>,
    pub spell_projectiles: Vec<SpellProjectileSnapshot>,
    pub spell_arcs: Vec<SpellArcSnapshot>,
}
