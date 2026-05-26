//! Compact binary snapshot format for multiplayer state synchronization.
//!
//! The host serializes the game state into a `GameSnapshot` each frame and
//! sends it over the unreliable WebRTC data channel. The guest deserializes
//! and renders ghost entities from the snapshot data.

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::game::units::components::{Health, Team};

use super::entity_map::NetworkEntityId;

/// Unit state snapshot sent from host to guest each frame.
///
/// Contains only unit and unit-projectile data. Spell visuals are sent
/// separately via `SpellVisualSnapshot` (bidirectional).
#[derive(Serialize, Deserialize)]
pub struct GameSnapshot {
    /// Monotonically increasing tick counter for ordering.
    pub tick: u32,
    /// State of every tracked unit.
    pub units: Vec<UnitSnapshot>,
    /// State of every in-flight arrow projectile.
    pub arrows: Vec<ArrowSnapshot>,
}

/// Per-unit state with CRDT health data (~37 bytes).
/// Per-unit network snapshot.
///
/// **Wire-format note:** bincode 1.x encodes fields positionally with no
/// version tags, so adding/removing/reordering any field is a breaking
/// change — mixed-version peers silently misread later fields as garbage
/// rather than failing fast. MP currently requires same-version peers.
/// If you ever need cross-version sessions, prefix the snapshot with a
/// protocol-version byte or move to a tagged encoding.
///
/// Velocity is XZ-only (`vx`, `vz`). The `Velocity` component itself has
/// no `y`, and animation systems read only XZ. If a future airborne ghost
/// type ever cares about vertical motion, add a `vy` field here and ship
/// it through `build_unit_snapshot`.
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
    /// Velocity X (host-authoritative, used by guest animation systems).
    pub vx: f32,
    /// Velocity Z (host-authoritative, used by guest animation systems).
    pub vz: f32,
    /// Team encoded as u8: 0=Defenders, 1=Attackers, 2=Undead.
    pub team: u8,
    /// Health as a 0-100 percentage (for visual rendering).
    pub health_pct: u8,
    /// Bitfield flags (see `UnitFlags`).
    pub flags: u8,
    /// Maximum HP for CRDT health calculation.
    pub max_hp: f32,
    /// CRDT damage counters per peer (monotonically increasing).
    pub damage: [f32; 2],
    /// CRDT healing counters per peer (monotonically increasing).
    pub healing: [f32; 2],
}

/// Bitfield constants for `UnitSnapshot::flags`.
pub struct UnitFlags;

impl UnitFlags {
    pub const CORPSE: u8 = 1 << 0;
    pub const KING: u8 = 1 << 1;
    pub const ARCHER: u8 = 1 << 2;
    pub const KINGS_GUARD: u8 = 1 << 3;
    pub const FIRE_EFFECT: u8 = 1 << 4;
    pub const FROST_EFFECT: u8 = 1 << 5;
    pub const ELECTRIC_EFFECT: u8 = 1 << 6;
    pub const SPELL_SHIELD: u8 = 1 << 7;
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
///
/// If the entity has a `CrdtHealth` component, its CRDT state is used directly.
/// Otherwise, we derive initial CRDT values from the `Health` component.
#[allow(clippy::too_many_arguments)]
pub fn build_unit_snapshot(
    net_id: &NetworkEntityId,
    transform: &Transform,
    velocity: &crate::game::components::Velocity,
    team: &Team,
    health: &Health,
    crdt_health: Option<&crate::networking::crdt::CrdtHealth>,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_kings_guard: bool,
    has_fire: bool,
    has_frost: bool,
    has_electric: bool,
    has_spell_shield: bool,
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
    if has_fire {
        flags |= UnitFlags::FIRE_EFFECT;
    }
    if has_frost {
        flags |= UnitFlags::FROST_EFFECT;
    }
    if has_electric {
        flags |= UnitFlags::ELECTRIC_EFFECT;
    }
    if has_spell_shield {
        flags |= UnitFlags::SPELL_SHIELD;
    }

    let (max_hp, damage, healing) = if let Some(crdt) = crdt_health {
        (crdt.max_hp, crdt.damage, crdt.healing)
    } else {
        // Derive from Health: all damage is attributed to peer 0 (host)
        let total_damage = (health.max - health.current).max(0.0);
        (health.max, [total_damage, 0.0], [0.0, 0.0])
    };

    UnitSnapshot {
        id: net_id.0,
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        vx: velocity.x,
        vz: velocity.z,
        team: team_to_u8(team),
        health_pct,
        flags,
        max_hp,
        damage,
        healing,
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

/// Type prefix bytes for unreliable channel messages.
///
/// Each unreliable message starts with a 1-byte prefix so receivers can
/// distinguish between different payload types.
pub const UNRELIABLE_GAME_SNAPSHOT: u8 = 0;
pub const UNRELIABLE_SPELL_SNAPSHOT: u8 = 1;
pub const UNRELIABLE_CRDT_SNAPSHOT: u8 = 2;

/// Spell visual data sent bidirectionally between host and guest.
///
/// Each client collects their local spell visuals (effects, projectiles, arcs,
/// missiles, beams) and sends them so the other client can render ghosts.
#[derive(Serialize, Deserialize, Default)]
pub struct SpellVisualSnapshot {
    /// Persistent spell effects (zones, walls, black holes, explosions, etc.).
    pub spell_effects: Vec<SpellEffectSnapshot>,
    /// Ephemeral spell projectiles (fireballs, ice, meteors in flight).
    pub spell_projectiles: Vec<SpellProjectileSnapshot>,
    /// Ephemeral spell arcs/beams (chain lightning, finger of death, etc.).
    pub spell_arcs: Vec<SpellArcSnapshot>,
    /// Magic missile positions.
    pub magic_missiles: Vec<MagicMissileSnapshot>,
    /// Beam positions (disintegrate, etc.).
    pub beams: Vec<BeamSnapshot>,
}

/// Compact CRDT health state sent from guest to host.
///
/// The guest sends only its local CRDT counters (damage/healing from peer 1)
/// so the host can merge them. Much smaller than a full GameSnapshot since
/// we only need the network ID + CRDT arrays per unit.
#[derive(Serialize, Deserialize)]
pub struct CrdtSnapshot {
    /// Per-unit CRDT health data.
    pub units: Vec<CrdtUnitUpdate>,
}

/// Per-unit CRDT health update (~21 bytes).
#[derive(Serialize, Deserialize)]
pub struct CrdtUnitUpdate {
    /// Network entity ID.
    pub id: u32,
    /// CRDT damage counters per peer.
    pub damage: [f32; 2],
    /// CRDT healing counters per peer.
    pub healing: [f32; 2],
    /// Status effect flags (FIRE_EFFECT, FROST_EFFECT, ELECTRIC_EFFECT bits from UnitFlags).
    pub effects: u8,
}

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
    /// 4=FingerOfDeath, 5=LightningRodArc, 6=Disintegrate.
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
