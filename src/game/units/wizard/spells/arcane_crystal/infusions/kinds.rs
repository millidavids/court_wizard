//! What a crystal has been infused with, and how that infusion is driven.
//!
//! One flat enum rather than a nested one: the run-condition factory in
//! [`super::run_conditions`] compares infusions by value, so every leaf needs its
//! own discriminant. [`CrystalInfusion::family`] is the single exhaustive match
//! over the enum — adding a variant without classifying it fails the build, which
//! is what keeps the dispatch in `auto/` and `infusions/` honest.

use bevy::prelude::*;

use super::super::constants::*;
use super::modifiers::CrystalModifier;
use crate::game::units::wizard::components::Spell;

/// What landing a spell on a crystal does to it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CrystalCharge {
    /// Replaces what the crystal projects.
    Infuse(CrystalInfusion),
    /// Replaces the infusion and layers a modifier on top.
    InfuseWithModifier(CrystalInfusion, CrystalModifier),
    /// Leaves the infusion alone and only layers a modifier — Haste, so a crystal
    /// already charged with a damage spell can be sped up rather than overwritten.
    ModifierOnly(CrystalModifier),
    /// Destroys the crystal in a burst scaled by its remaining lifetime.
    Shatter,
}

/// How an infusion gets driven each frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum InfusionFamily {
    /// One of the six legacy projectile echoes, driven by `auto_cast_remembered_spell`.
    Emitter,
    /// Driven by its own system on [`CrystalInfusion::interval`].
    Ticked,
    /// Driven by its own system every frame; `interval()` is meaningless (0.0).
    Continuous,
}

/// The spell a crystal has absorbed and now projects for the rest of its life.
///
/// Mutually exclusive — a new absorption replaces the previous one. Orthogonal to
/// [`super::modifiers`], which layer on top and change how the crystal itself behaves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CrystalInfusion {
    // === Emitters (the original six) ===
    MagicMissile,
    Fireball,
    ChainLightning,
    Meteor,
    FingerOfDeath,
    Disintegrate,

    // === Utility ===
    LightningRod,
    RaiseTheDead,
    Telekinesis,

    // === Zones / hazards ===
    Grease,
    Squall,
    SpikeGrowth,
    PlagueWind,

    // === Support auras ===
    BattleHymn,
    BerserkerRage,
    GuardianCircle,
    HealingPlume,
    MarkOfDeath,

    // === Control ===
    Entangle,
    Sleep,
    FogCloud,
    Banishment,

    // === Exotic ===
    Teleport,
    BlackHole,
}

impl CrystalInfusion {
    /// Classifies how this infusion is driven.
    ///
    /// This is the one exhaustive match over [`CrystalInfusion`]. Every other
    /// dispatch site gates on the family first, so a new variant surfaces here as a
    /// compile error rather than silently doing nothing.
    pub(crate) fn family(self) -> InfusionFamily {
        match self {
            Self::MagicMissile
            | Self::Fireball
            | Self::ChainLightning
            | Self::Meteor
            | Self::FingerOfDeath
            | Self::Disintegrate => InfusionFamily::Emitter,

            Self::Grease
            | Self::SpikeGrowth
            | Self::PlagueWind
            | Self::RaiseTheDead
            | Self::GuardianCircle
            | Self::Entangle
            | Self::Sleep
            | Self::FogCloud
            | Self::Banishment
            | Self::Teleport
            // Refreshed on a timer, not per frame. These hand out timed buffs, so
            // re-applying every frame would re-insert the modifier and every
            // talent component ~60x a second per unit — and Battle Hymn credits
            // talent progress on each call, which would unlock its tree in
            // seconds. The refresh beat sits well inside the buff duration, so
            // the aura still reads as continuous.
            | Self::BattleHymn
            | Self::BerserkerRage
            // Ticked, not continuous: a frame-by-frame vacuum would take every
            // drop the instant it landed, and Telekinesis cannot even begin
            // casting without an un-collected drop near the cursor — the
            // infusion would disable the spell that created it.
            | Self::Telekinesis => InfusionFamily::Ticked,

            // These keep a single long-lived source entity alive rather than
            // re-emitting: the crystal *becomes* a rod / a storm.
            // Genuinely per-frame: these either hold one source entity alive
            // (idempotent) or apply a force that must be integrated every tick.
            Self::LightningRod
            | Self::Squall
            | Self::HealingPlume
            | Self::MarkOfDeath
            | Self::BlackHole => InfusionFamily::Continuous,
        }
    }

    /// What a spell does to a crystal it lands on.
    ///
    /// Exhaustive over [`Spell`], so adding a spell to the game forces a decision
    /// here rather than silently producing an inert interaction.
    pub(crate) fn from_spell(spell: Spell) -> Option<CrystalCharge> {
        use CrystalCharge as C;
        Some(match spell {
            // --- Projectile echoes, detected by polling in `hits/` rather than
            // --- through the area-cast message. Listed for completeness.
            Spell::MagicMissile => C::Infuse(Self::MagicMissile),
            Spell::Fireball => C::Infuse(Self::Fireball),
            Spell::ChainLightning => C::Infuse(Self::ChainLightning),
            Spell::MeteorFall => C::Infuse(Self::Meteor),
            Spell::FingerOfDeath => C::Infuse(Self::FingerOfDeath),
            Spell::Disintegrate => C::Infuse(Self::Disintegrate),

            // --- Infusions ---
            Spell::LightningRod => C::Infuse(Self::LightningRod),
            Spell::RaiseTheDead => C::Infuse(Self::RaiseTheDead),
            Spell::Telekinesis => C::Infuse(Self::Telekinesis),
            Spell::Grease => C::Infuse(Self::Grease),
            Spell::Squall => C::Infuse(Self::Squall),
            Spell::SpikeGrowth => C::Infuse(Self::SpikeGrowth),
            Spell::PlagueWind => C::Infuse(Self::PlagueWind),
            Spell::BattleHymn => C::Infuse(Self::BattleHymn),
            Spell::HealingPlume => C::Infuse(Self::HealingPlume),
            Spell::MarkOfDeath => C::Infuse(Self::MarkOfDeath),
            Spell::Entangle => C::Infuse(Self::Entangle),
            Spell::Sleep => C::Infuse(Self::Sleep),
            Spell::FogCloud => C::Infuse(Self::FogCloud),
            Spell::Banishment => C::Infuse(Self::Banishment),
            Spell::Teleport => C::Infuse(Self::Teleport),
            Spell::BlackHole => C::Infuse(Self::BlackHole),

            // --- Infusion plus a modifier ---
            Spell::BerserkerRage => {
                C::InfuseWithModifier(Self::BerserkerRage, CrystalModifier::Enraged)
            }
            Spell::GuardianCircle => {
                C::InfuseWithModifier(Self::GuardianCircle, CrystalModifier::Warded)
            }
            // --- Modifier only: deliberately does not replace the infusion, so a
            // --- crystal can be charged with a damage spell and then hasted or
            // --- rooted against black-hole gravity.
            Spell::Haste => C::ModifierOnly(CrystalModifier::Hastened),
            Spell::WallOfStone => C::ModifierOnly(CrystalModifier::Anchored),

            // --- Shatters the crystal instead of charging it ---
            Spell::Dispel => C::Shatter,

            // --- No interaction ---
            // A crystal cannot charge itself. The other three have designed
            // interactions that are not built yet; they deliberately return
            // `None` rather than setting an infusion with no system behind it,
            // which would leave the player with a crystal that does nothing.
            Spell::ArcaneCrystal | Spell::Polymorph | Spell::MindControl | Spell::WallOfFire => {
                return None;
            }
        })
    }

    /// Stable numeric id for the multiplayer snapshot. `0` means "no infusion",
    /// so ids start at 1. Append new variants at the end — renumbering would
    /// desync peers running different builds.
    pub(crate) fn as_sync_id(infusion: Option<Self>) -> f32 {
        let Some(infusion) = infusion else {
            return 0.0;
        };
        (Self::ALL
            .iter()
            .position(|candidate| *candidate == infusion)
            .unwrap_or(usize::MAX)
            .wrapping_add(1)) as f32
    }

    /// Inverse of [`Self::as_sync_id`].
    pub(crate) fn from_sync_id(id: f32) -> Option<Self> {
        let index = id.round() as i64;
        if index <= 0 {
            return None;
        }
        Self::ALL.get((index - 1) as usize).copied()
    }

    /// Every variant, in snapshot-id order.
    pub(crate) const ALL: [Self; 24] = [
        Self::MagicMissile,
        Self::Fireball,
        Self::ChainLightning,
        Self::Meteor,
        Self::FingerOfDeath,
        Self::Disintegrate,
        Self::LightningRod,
        Self::RaiseTheDead,
        Self::Telekinesis,
        Self::Grease,
        Self::Squall,
        Self::SpikeGrowth,
        Self::PlagueWind,
        Self::BattleHymn,
        Self::BerserkerRage,
        Self::GuardianCircle,
        Self::HealingPlume,
        Self::MarkOfDeath,
        Self::Entangle,
        Self::Sleep,
        Self::FogCloud,
        Self::Banishment,
        Self::Teleport,
        Self::BlackHole,
    ];

    /// Base number of sub-effects a burst produces before count and echo scaling.
    pub(crate) fn base_count(self) -> usize {
        match self {
            Self::MagicMissile => MINI_MISSILE_COUNT,
            Self::Fireball => MINI_FB_COUNT,
            Self::Meteor => MINI_METEOR_COUNT,
            Self::ChainLightning => LIGHTNING_ARC_COUNT,
            Self::FingerOfDeath | Self::Disintegrate => BEAM_COUNT,
            _ => INFUSION_BURST_COUNT,
        }
    }

    /// Representative damage for emitter cadence. Meaningless for other families.
    fn emitter_damage_value(self) -> f32 {
        match self {
            Self::MagicMissile => 5.0,
            Self::ChainLightning => 20.0,
            Self::Meteor => 25.0,
            Self::Fireball => 50.0,
            Self::FingerOfDeath => 1000.0,
            // Constant beam — no interval.
            _ => 0.0,
        }
    }

    /// Seconds between activations. `0.0` means continuous (no timer at all):
    /// both the Disintegrate beam and every [`InfusionFamily::Continuous`] infusion.
    pub(crate) fn interval(self) -> f32 {
        match self.family() {
            InfusionFamily::Continuous => 0.0,
            InfusionFamily::Emitter => {
                if self == Self::Disintegrate {
                    return 0.0;
                }
                let raw = AUTO_CAST_BASE_INTERVAL
                    * (self.emitter_damage_value() / AUTO_CAST_REFERENCE_DAMAGE);
                raw.clamp(AUTO_CAST_MIN_INTERVAL, AUTO_CAST_MAX_INTERVAL)
            }
            InfusionFamily::Ticked => match self {
                Self::Grease => GREASE_INFUSION_INTERVAL,
                Self::SpikeGrowth => SPIKE_GROWTH_INFUSION_INTERVAL,
                Self::PlagueWind => PLAGUE_WIND_INFUSION_INTERVAL,
                Self::RaiseTheDead => RAISE_THE_DEAD_INFUSION_INTERVAL,
                Self::GuardianCircle => GUARDIAN_CIRCLE_INFUSION_INTERVAL,
                Self::Entangle => ENTANGLE_INFUSION_INTERVAL,
                Self::Sleep => SLEEP_INFUSION_INTERVAL,
                Self::FogCloud => FOG_CLOUD_INFUSION_INTERVAL,
                Self::Banishment => BANISHMENT_INFUSION_INTERVAL,
                Self::Teleport => TELEPORT_INFUSION_INTERVAL,
                Self::Telekinesis => TELEKINESIS_INFUSION_INTERVAL,
                Self::BattleHymn | Self::BerserkerRage => AURA_REFRESH_INTERVAL,
                // Unreachable: family() already narrowed to the Ticked set above.
                _ => AUTO_CAST_BASE_INTERVAL,
            },
        }
    }

    /// Emissive tint for the crystal body while this infusion is held. Gives the
    /// player a read on what the crystal currently is without a UI element.
    pub(crate) fn color(self) -> LinearRgba {
        match self {
            // Fire
            Self::Fireball | Self::Meteor | Self::Disintegrate => {
                LinearRgba::new(2.4, 0.7, 0.15, 1.0)
            }
            // Electric
            Self::ChainLightning | Self::LightningRod => LinearRgba::new(0.9, 1.8, 2.6, 1.0),
            // Frost
            Self::Squall | Self::FogCloud => LinearRgba::new(0.9, 1.7, 2.2, 1.0),
            // Necrotic
            Self::FingerOfDeath | Self::RaiseTheDead | Self::MarkOfDeath => {
                LinearRgba::new(0.6, 1.5, 0.5, 1.0)
            }
            // Nature / poison
            Self::Grease | Self::SpikeGrowth | Self::PlagueWind | Self::Entangle => {
                LinearRgba::new(0.5, 1.6, 0.4, 1.0)
            }
            // Support
            Self::HealingPlume | Self::GuardianCircle | Self::BattleHymn => {
                LinearRgba::new(2.2, 1.9, 0.8, 1.0)
            }
            Self::BerserkerRage => LinearRgba::new(2.6, 0.4, 0.3, 1.0),
            // Force / arcane — also the crystal's default violet
            Self::MagicMissile
            | Self::Telekinesis
            | Self::Sleep
            | Self::Banishment
            | Self::Teleport
            | Self::BlackHole => CRYSTAL_DEFAULT_EMISSIVE,
        }
    }
}
