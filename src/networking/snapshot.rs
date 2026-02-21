//! Compact binary snapshot format for multiplayer state synchronization.
//!
//! The host serializes the game state into a `GameSnapshot` each frame and
//! sends it over the unreliable WebRTC data channel. The guest deserializes
//! and renders ghost entities from the snapshot data.

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::game::units::components::{Health, Team};

use super::entity_map::NetworkEntityId;

/// Complete game state snapshot sent from host to guest each frame.
#[derive(Serialize, Deserialize)]
pub struct GameSnapshot {
    /// Monotonically increasing tick counter for ordering.
    pub tick: u32,
    /// State of every tracked unit.
    pub units: Vec<UnitSnapshot>,
    /// State of every in-flight arrow projectile.
    pub arrows: Vec<ArrowSnapshot>,
    /// State of every in-flight magic missile.
    pub magic_missiles: Vec<MagicMissileSnapshot>,
    /// State of every active beam (disintegrate, etc.).
    pub beams: Vec<BeamSnapshot>,
    /// Persistent spell effects (zones, walls, black holes, explosions, etc.).
    pub spell_effects: Vec<SpellEffectSnapshot>,
    /// Ephemeral spell projectiles (fireballs, ice, meteors in flight).
    pub spell_projectiles: Vec<SpellProjectileSnapshot>,
    /// Ephemeral spell arcs/beams (chain lightning, finger of death, etc.).
    pub spell_arcs: Vec<SpellArcSnapshot>,
}

/// Compact per-unit state (~21 bytes).
#[derive(Serialize, Deserialize)]
pub struct UnitSnapshot {
    /// Network entity ID assigned by the host.
    pub id: u32,
    /// World position X.
    pub x: f32,
    /// World position Y (height above battlefield).
    pub y: f32,
    /// World position Z.
    pub z: f32,
    /// Team encoded as u8: 0=Defenders, 1=Attackers, 2=Undead.
    pub team: u8,
    /// Health as a 0-100 percentage.
    pub health_pct: u8,
    /// Bitfield flags (see `UnitFlags`).
    pub flags: u8,
}

/// Bitfield constants for `UnitSnapshot::flags`.
pub struct UnitFlags;

impl UnitFlags {
    pub const CORPSE: u8 = 1 << 0;
    pub const KING: u8 = 1 << 1;
    pub const ARCHER: u8 = 1 << 2;
    pub const KINGS_GUARD: u8 = 1 << 3;
}

/// Encodes a `Team` component into a u8.
pub fn team_to_u8(team: &Team) -> u8 {
    match team {
        Team::Defenders => 0,
        Team::Attackers => 1,
        Team::Undead => 2,
    }
}

/// Decodes a u8 back into a `Team`.
pub fn u8_to_team(val: u8) -> Team {
    match val {
        0 => Team::Defenders,
        1 => Team::Attackers,
        _ => Team::Undead,
    }
}

/// Builds a `UnitSnapshot` from an entity's components.
#[allow(clippy::too_many_arguments)]
pub fn build_unit_snapshot(
    net_id: &NetworkEntityId,
    transform: &Transform,
    team: &Team,
    health: &Health,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_kings_guard: bool,
) -> UnitSnapshot {
    let health_pct = if health.max > 0.0 {
        ((health.current / health.max) * 100.0).clamp(0.0, 100.0) as u8
    } else {
        0
    };

    let mut flags = 0u8;
    if is_corpse {
        flags |= UnitFlags::CORPSE;
    }
    if is_king {
        flags |= UnitFlags::KING;
    }
    if is_archer {
        flags |= UnitFlags::ARCHER;
    }
    if is_kings_guard {
        flags |= UnitFlags::KINGS_GUARD;
    }

    UnitSnapshot {
        id: net_id.0,
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        team: team_to_u8(team),
        health_pct,
        flags,
    }
}

/// Compact per-arrow state (~12 bytes).
#[derive(Serialize, Deserialize)]
pub struct ArrowSnapshot {
    /// World position X.
    pub x: f32,
    /// World position Y (height).
    pub y: f32,
    /// World position Z.
    pub z: f32,
}

/// Compact per-magic-missile state (~12 bytes).
#[derive(Serialize, Deserialize)]
pub struct MagicMissileSnapshot {
    /// World position X.
    pub x: f32,
    /// World position Y (height).
    pub y: f32,
    /// World position Z.
    pub z: f32,
}

/// Compact per-beam state (~28 bytes).
///
/// Encodes origin, direction, and length for disintegrate and similar beams.
#[derive(Serialize, Deserialize)]
pub struct BeamSnapshot {
    /// Origin X.
    pub ox: f32,
    /// Origin Y.
    pub oy: f32,
    /// Origin Z.
    pub oz: f32,
    /// Direction X.
    pub dx: f32,
    /// Direction Y.
    pub dy: f32,
    /// Direction Z.
    pub dz: f32,
    /// Beam length.
    pub length: f32,
}

/// Monotonically increasing tick counter for snapshot ordering.
#[derive(Resource, Default)]
pub struct SnapshotTick(pub u32);

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
            _ => Err(()),
        }
    }
}

/// Persistent spell effect snapshot (~40 bytes).
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
}

/// Ephemeral spell projectile snapshot (~13 bytes).
///
/// Despawned and re-spawned each frame on the guest, like arrows.
#[derive(Serialize, Deserialize)]
pub struct SpellProjectileSnapshot {
    /// Projectile type: 0=Fireball, 1=IceProjectile, 2=MeteorProjectile.
    pub kind: u8,
    /// World position X.
    pub x: f32,
    /// World position Y.
    pub y: f32,
    /// World position Z.
    pub z: f32,
}

/// Ephemeral spell arc/beam snapshot (~25 bytes).
///
/// Despawned and re-spawned each frame on the guest.
#[derive(Serialize, Deserialize)]
pub struct SpellArcSnapshot {
    /// Arc type: 0=ChainLightning, 1=LightningStrike, 2=CrystalBeam,
    /// 3=CrystalLightning, 4=FingerOfDeath, 5=LightningRodArc.
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
