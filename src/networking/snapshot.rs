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
    /// Host's authoritative match-elapsed seconds (`KillStats.elapsed_time`), so
    /// the guest's HUD clock mirrors the host exactly instead of free-running on
    /// its own local timer. Appended last (positional bincode; same-version MP).
    pub host_elapsed_secs: f32,
}

/// Per-unit state with CRDT health data (~37 bytes).
/// Per-unit network snapshot.
///
/// **HP via CRDT, not via `health_pct`:** the CRDT damage[2]/healing[2]
/// slots are the source of truth for HP — they support both peers
/// applying damage AND healing to the same unit and converging via
/// element-wise max merge. `Health.current` is re-derived from CRDT on
/// each peer. The previous `health_pct` field is dropped because it was
/// redundant with the CRDT-derived value.
///
/// **Wire-format note:** bincode 1.x encodes fields positionally with no
/// version tags, so adding/removing/reordering any field is a breaking
/// change — mixed-version peers silently misread later fields as garbage
/// rather than failing fast. MP currently requires same-version peers.
///
/// Velocity is XZ-only (`vx`, `vz`).
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
    /// Bitfield flags (see `UnitFlags`). u16 because the original u8 ran
    /// out of bits when COMBAT_ANIMATION was added.
    pub flags: u16,
    /// Max HP — needed once but ships every frame (CRDT seed; could be
    /// thinned later via a separate one-shot spawn message).
    pub max_hp: f32,
    /// CRDT damage counters per peer (monotonically increasing). Both
    /// peers apply damage to their local Health → `sync_health_to_crdt`
    /// records into their own slot → snapshots cross-fertilise → merge
    /// = element-wise max so both sides converge.
    pub damage: [f32; 2],
    /// CRDT healing counters per peer (monotonically increasing).
    pub healing: [f32; 2],
}

/// Bitfield constants for `UnitSnapshot::flags`.
pub struct UnitFlags;

impl UnitFlags {
    pub const CORPSE: u16 = 1 << 0;
    pub const KING: u16 = 1 << 1;
    pub const ARCHER: u16 = 1 << 2;
    pub const KINGS_GUARD: u16 = 1 << 3;
    pub const FIRE_EFFECT: u16 = 1 << 4;
    pub const FROST_EFFECT: u16 = 1 << 5;
    pub const ELECTRIC_EFFECT: u16 = 1 << 6;
    pub const SPELL_SHIELD: u16 = 1 << 7;
    /// Host's unit currently has a `CombatAnimation` component — set so
    /// the guest can spawn the matching swing/shoot animation on its
    /// ghost. Without this the ghost stays on the idle frame even though
    /// the host's unit is actively swinging in melee.
    pub const COMBAT_ANIMATION: u16 = 1 << 8;
    /// Host's unit currently carries Mark of Death — set so the guest renders
    /// the floating mark indicator on its ghost unit. Without this, marks the
    /// HOST casts never reach the guest (the guest only knows about marks it
    /// cast itself and applied locally).
    pub const MARK_EFFECT: u16 = 1 << 9;
    /// Host's unit is poisoned (Plague Wind etc.) — set so the guest renders
    /// the green poison tint on its ghost unit.
    pub const POISON_EFFECT: u16 = 1 << 10;
    /// Host's unit is currently polymorphed (a sheep) — set so the guest swaps
    /// its ghost to the sheep sprite. Without this a host-cast polymorph never
    /// renders on the guest (the unit snapshot otherwise carries no "is a sheep"
    /// state and the guest only swaps materials at spawn / corpse transitions).
    pub const POLYMORPH: u16 = 1 << 11;
    /// Host's unit IS the Swordcerer battlefield avatar — set so the guest spawns
    /// the ghost with the avatar sprite/hitbox and renders its health bar, rather
    /// than treating it as a generic infantry unit.
    pub const SWORDCERER_AVATAR: u16 = 1 << 12;
    /// Host's unit is currently smelly (Excremage poop debuff) — set so the guest
    /// renders the brown stink tint on its ghost unit.
    pub const SMELLY: u16 = 1 << 13;
    /// Host's unit is engaged in melee — set so the guest can run the battle
    /// ambience (melee-sound) loop scaled by the on-field combat it can hear.
    pub const IN_MELEE: u16 = 1 << 14;
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
/// HP rides in the CRDT damage/healing arrays (NOT a redundant `health_pct`)
/// so both peers can apply damage AND healing and converge via element-wise
/// max. `health_pct` is gone — guest derives `Health.current` from CRDT.
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
    has_combat_animation: bool,
    has_mark: bool,
    has_poison: bool,
    has_polymorph: bool,
    has_smelly: bool,
    has_swordcerer_avatar: bool,
    has_in_melee: bool,
) -> UnitSnapshot {
    let mut flags = 0u16;
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
    if has_combat_animation {
        flags |= UnitFlags::COMBAT_ANIMATION;
    }
    if has_mark {
        flags |= UnitFlags::MARK_EFFECT;
    }
    if has_poison {
        flags |= UnitFlags::POISON_EFFECT;
    }
    if has_polymorph {
        flags |= UnitFlags::POLYMORPH;
    }
    if has_smelly {
        flags |= UnitFlags::SMELLY;
    }
    if has_swordcerer_avatar {
        flags |= UnitFlags::SWORDCERER_AVATAR;
    }
    if has_in_melee {
        flags |= UnitFlags::IN_MELEE;
    }

    let (max_hp, damage, healing) = if let Some(crdt) = crdt_health {
        (crdt.max_hp, crdt.damage, crdt.healing)
    } else {
        // First snapshot before `attach_crdt_health` has run — derive
        // initial CRDT state from `Health` (all loss attributed to peer 0).
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
    /// Beam core width (`DisintegrateBeam::beam_width()`) so the ghost renders at
    /// the caster's real thickness instead of a hardcoded value.
    pub width: f32,
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
    /// One-shot cast VFX events fired this tick (school flares, aura
    /// bubbles, smoke poofs, motes, sparks, dust). The receiver iterates
    /// these and spawns the matching local VFX via the existing
    /// `vfx::systems::spawn_*` helpers. Drained once per send tick.
    pub cast_events: Vec<CastEventSnapshot>,
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

// ============================================================================
// Cast-event snapshots — one-shot VFX synced to the remote peer
// ============================================================================
//
// One-shot cast VFX (school flares, aura bubbles, smoke poofs, floating motes,
// sparks, dust) are spawned imperatively in casting handlers, not driven by
// persistent component state, so the existing snapshot infrastructure can't
// cover them. Instead each cast emits a small `CastEventSnapshot` into the
// outgoing `SpellVisualSnapshot.cast_events`; the receiver iterates them and
// spawns the matching local VFX via `vfx::systems::spawn_*`.

/// Kind of one-shot cast VFX event. `subkind` semantics depend on the kind.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum CastEventKind {
    /// School cast flare (`spawn_school_flare`). `subkind` = `SpellSchool` ordinal.
    SchoolFlare = 0,
    /// Expanding aura bubble (`spawn_aura_bubble`). `subkind` = `AuraBubbleVariant`,
    /// `extra` = `[radius, duration, 0, 0]`.
    AuraBubble = 1,
    /// Contracting aura bubble (`spawn_aura_bubble_contracting`). Same payload
    /// as `AuraBubble`.
    AuraBubbleContract = 2,
    /// Smoke poof (`spawn_smoke_poof`). `subkind` = `PoofVariant`.
    SmokePoof = 3,
    /// Floating motes (`spawn_floating_motes`). `subkind` = `MoteMaterial`,
    /// `extra` = `[radius, count_as_f32, 0, 0]`.
    FloatingMotes = 4,
    /// Spark burst (`spawn_sparks_with_material`). `subkind` = `SparkMaterial`.
    Sparks = 5,
    /// Generic dust smoke (`spawn_dust_smoke`).
    DustSmoke = 6,
    /// Banishment lensing sphere (`BanishmentVfx` component).
    /// `extra` = `[radius, duration, 0, 0]`.
    BanishmentLens = 7,
    /// Final Stand explosion sphere (`FinalStandExplosionVfx` component) —
    /// Berserker Rage's Final Stand talent detonates the unit on death.
    /// `extra` = `[max_radius, lifetime, 0, 0]`.
    FinalStandExplosion = 8,
    /// One-shot spell SFX. `subkind` = `SpellSoundId` ordinal; `x/y/z` = world
    /// position for distance attenuation; `extra[0]` = volume scale.
    SfxOneShot = 9,
    /// Warglock gun muzzle flash. `x/y/z` = muzzle origin; `subkind` = `GunType`
    /// ordinal; `extra[0]` = flash radius.
    GunMuzzleFlash = 10,
    /// Warglock hitscan bullet tracer. `x/y/z` = origin; `subkind` = `GunType`
    /// ordinal; `extra` = `[dir_x, dir_y, dir_z, length]`.
    GunBulletTracer = 11,
    /// Warglock flamethrower flame particle. `x/y/z` = spawn position;
    /// `extra` = `[vel_x, vel_y, vel_z, lifetime]`.
    GunFlameParticle = 12,
    /// Swordcerer sword-swing arc. `x/y/z` = avatar position; `extra` =
    /// `[dir_x, dir_z, 0, 0]` (swing facing).
    SwordArc = 13,
}

impl TryFrom<u8> for CastEventKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SchoolFlare),
            1 => Ok(Self::AuraBubble),
            2 => Ok(Self::AuraBubbleContract),
            3 => Ok(Self::SmokePoof),
            4 => Ok(Self::FloatingMotes),
            5 => Ok(Self::Sparks),
            6 => Ok(Self::DustSmoke),
            7 => Ok(Self::BanishmentLens),
            8 => Ok(Self::FinalStandExplosion),
            9 => Ok(Self::SfxOneShot),
            10 => Ok(Self::GunMuzzleFlash),
            11 => Ok(Self::GunBulletTracer),
            12 => Ok(Self::GunFlameParticle),
            13 => Ok(Self::SwordArc),
            _ => Err(()),
        }
    }
}

/// Identifies a one-shot spell sound effect for cross-client playback.
/// Carried in `CastEventSnapshot::subkind` when `kind == SfxOneShot`. The
/// receiver maps each variant back to a handle in `SpellSfxAssets` via the
/// audio module's `lookup_sfx_handle`. Adding a new variant requires a
/// matching arm there and a matching `TryFrom<u8>` entry below.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellSoundId {
    MagicMissileCast = 0,
    FireballCast = 1,
    FireballImpact = 2,
    ArcaneCrystalCast = 3,
    BanishmentCast = 4,
    BattleHymnCast = 5,
    BerserkerRageCast = 6,
    ChainLightningCast = 7,
    HealingPlumeCast = 8,
    DispelCast = 9,
    EntangleCast = 10,
    FingerOfDeathCast = 11,
    FogCloudCast = 12,
    GreaseCast = 13,
    GuardianCircleCast = 14,
    HasteCast = 15,
    LightningRodImpact = 16,
    MarkOfDeathCast = 17,
    MindControlCast = 18,
    PlagueWindCast = 19,
    PolymorphCast = 20,
    RaiseTheDeadCast = 21,
    SleepCast = 22,
    SpikeGrowthCast = 23,
    SquallImpact = 24,
    TelekinesisCast = 25,
    TeleportCast = 26,
    WallOfStoneCast = 27,
    BoulderImpact = 28,
    RayEyeDeath = 29,
    /// Looping disintegrate beam channel sound (played one-shot on the remote peer).
    DisintegrateChannel = 30,
    // (Excremage fart is applied by the receiver substituting the handle for a
    // remote Excremage caster — no dedicated sound id needed.)
    /// Warglock gun shots.
    MachineGunShot = 31,
    MagnumShot = 32,
    ShotgunShot = 33,
    RocketShot = 34,
    FlamethrowerBurst = 35,
    /// Meteorologist storm lightning strike — played on both peers.
    WeatherLightningStrike = 36,
}

impl TryFrom<u8> for SpellSoundId {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MagicMissileCast),
            1 => Ok(Self::FireballCast),
            2 => Ok(Self::FireballImpact),
            3 => Ok(Self::ArcaneCrystalCast),
            4 => Ok(Self::BanishmentCast),
            5 => Ok(Self::BattleHymnCast),
            6 => Ok(Self::BerserkerRageCast),
            7 => Ok(Self::ChainLightningCast),
            8 => Ok(Self::HealingPlumeCast),
            9 => Ok(Self::DispelCast),
            10 => Ok(Self::EntangleCast),
            11 => Ok(Self::FingerOfDeathCast),
            12 => Ok(Self::FogCloudCast),
            13 => Ok(Self::GreaseCast),
            14 => Ok(Self::GuardianCircleCast),
            15 => Ok(Self::HasteCast),
            16 => Ok(Self::LightningRodImpact),
            17 => Ok(Self::MarkOfDeathCast),
            18 => Ok(Self::MindControlCast),
            19 => Ok(Self::PlagueWindCast),
            20 => Ok(Self::PolymorphCast),
            21 => Ok(Self::RaiseTheDeadCast),
            22 => Ok(Self::SleepCast),
            23 => Ok(Self::SpikeGrowthCast),
            24 => Ok(Self::SquallImpact),
            25 => Ok(Self::TelekinesisCast),
            26 => Ok(Self::TeleportCast),
            27 => Ok(Self::WallOfStoneCast),
            28 => Ok(Self::BoulderImpact),
            29 => Ok(Self::RayEyeDeath),
            30 => Ok(Self::DisintegrateChannel),
            31 => Ok(Self::MachineGunShot),
            32 => Ok(Self::MagnumShot),
            33 => Ok(Self::ShotgunShot),
            34 => Ok(Self::RocketShot),
            35 => Ok(Self::FlamethrowerBurst),
            36 => Ok(Self::WeatherLightningStrike),
            _ => Err(()),
        }
    }
}

/// Spell school for `CastEventKind::SchoolFlare`. Matches the SP
/// `SpellSchool` enum 1:1 — keep ordinals in sync.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellSchoolWire {
    Fire = 0,
    Lightning = 1,
    Arcane = 2,
    Nature = 3,
    Holy = 4,
    Dark = 5,
    Force = 6,
    Transmutation = 7,
}

impl TryFrom<u8> for SpellSchoolWire {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Fire),
            1 => Ok(Self::Lightning),
            2 => Ok(Self::Arcane),
            3 => Ok(Self::Nature),
            4 => Ok(Self::Holy),
            5 => Ok(Self::Dark),
            6 => Ok(Self::Force),
            7 => Ok(Self::Transmutation),
            _ => Err(()),
        }
    }
}

/// Aura sphere material variant — picks one of the named handles in
/// `SpellVisualAssets`. Receiver maps each variant to the matching asset.
/// Only variants actually emitted by a `_synced` casting handler are kept
/// here; adding a new entry requires a matching `aura_material_handle`
/// dispatch arm in `spell_sync::apply_remote_cast_events`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum AuraBubbleVariant {
    Guardian = 0,
    BattleHymn = 1,
    Haste = 2,
    Berserker = 3,
    Sleep = 4,
    RaiseDead = 5,
    Teleport = 6,
}

impl TryFrom<u8> for AuraBubbleVariant {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Guardian),
            1 => Ok(Self::BattleHymn),
            2 => Ok(Self::Haste),
            3 => Ok(Self::Berserker),
            4 => Ok(Self::Sleep),
            5 => Ok(Self::RaiseDead),
            6 => Ok(Self::Teleport),
            _ => Err(()),
        }
    }
}

/// Smoke poof material variant.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum PoofVariant {
    Banishment = 0,
    Polymorph = 1,
}

impl TryFrom<u8> for PoofVariant {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Banishment),
            1 => Ok(Self::Polymorph),
            _ => Err(()),
        }
    }
}

/// Mote material variant for `spawn_floating_motes`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum MoteMaterial {
    Healing = 0,
    Nature = 1,
    Sleep = 2,
}

impl TryFrom<u8> for MoteMaterial {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Healing),
            1 => Ok(Self::Nature),
            2 => Ok(Self::Sleep),
            _ => Err(()),
        }
    }
}

/// Spark material variant for `spawn_sparks_with_material`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum SparkMaterial {
    Banishment = 0,
    Dispel = 1,
}

impl TryFrom<u8> for SparkMaterial {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Banishment),
            1 => Ok(Self::Dispel),
            _ => Err(()),
        }
    }
}

/// One-shot cast VFX event (~24 bytes on the wire).
///
/// Sent in the bidirectional `SpellVisualSnapshot.cast_events` vector. The
/// receiver dispatches each event on `kind` and spawns the matching local
/// VFX entity, tagged `OnGameplayScreen` so MP cleanup catches it (the
/// `cleanup_game` system runs on `OnExit(AppState::MultiplayerGame)`).
#[derive(Serialize, Deserialize)]
pub struct CastEventSnapshot {
    /// Event type (`CastEventKind` as u8).
    pub kind: u8,
    /// Kind-specific sub-discriminator (school, aura variant, poof variant, etc.).
    pub subkind: u8,
    /// World position X (typically the local wizard's `LocalSpellOrigin` or
    /// the cast target's circle indicator position).
    pub x: f32,
    /// World position Y.
    pub y: f32,
    /// World position Z.
    pub z: f32,
    /// Kind-specific extra payload (radius, duration, count, etc.). Most
    /// kinds use zero or one entry; reserved space avoids future churn.
    pub extra: [f32; 4],
}
