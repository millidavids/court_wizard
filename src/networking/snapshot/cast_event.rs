//! One-shot cast VFX event snapshot types.

use serde::{Deserialize, Serialize};

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
