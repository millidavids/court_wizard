use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::units::DamageType;

/// Available spells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub enum Spell {
    MagicMissile,
    Disintegrate,
    Fireball,
    GuardianCircle,
    ChainLightning,
    FingerOfDeath,
    RaiseTheDead,
    Teleport,
    WallOfStone,
    BlackHole,
    Squall,
    WallOfFire,
    Entangle,
    Haste,
    SpikeGrowth,
    LightningRod,
    Telekinesis,
    MeteorFall,
    MarkOfDeath,
    PlagueWind,
    HypnoticPattern,
    Sleep,
    Grease,
    FogCloud,
    BattleHymn,
    HealingPlume,
    BerserkerRage,
    PhantasmalForce,
    Banishment,
    Polymorph,
    ArcaneCrystal,
}

/// Spell categories for organizing the spell book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellCategory {
    Offense,
    Control,
    Support,
    Utility,
}

impl SpellCategory {
    /// Returns all categories in display order.
    pub const fn all() -> &'static [SpellCategory] {
        &[
            SpellCategory::Offense,
            SpellCategory::Control,
            SpellCategory::Support,
            SpellCategory::Utility,
        ]
    }

    /// Returns the display name for this category.
    pub const fn display_name(&self) -> &'static str {
        match self {
            SpellCategory::Offense => "Offense",
            SpellCategory::Control => "Control",
            SpellCategory::Support => "Support",
            SpellCategory::Utility => "Utility",
        }
    }

    /// Returns all spells in this category, in display order.
    pub const fn spells(&self) -> &'static [Spell] {
        match self {
            SpellCategory::Offense => &[
                Spell::MagicMissile,
                Spell::Disintegrate,
                Spell::Fireball,
                Spell::ChainLightning,
                Spell::FingerOfDeath,
                Spell::LightningRod,
                Spell::MeteorFall,
                Spell::MarkOfDeath,
                Spell::PlagueWind,
            ],
            SpellCategory::Control => &[
                Spell::BlackHole,
                Spell::WallOfStone,
                Spell::WallOfFire,
                Spell::Entangle,
                Spell::SpikeGrowth,
                Spell::Squall,
                Spell::HypnoticPattern,
                Spell::Sleep,
                Spell::Grease,
                Spell::Polymorph,
            ],
            SpellCategory::Support => &[
                Spell::GuardianCircle,
                Spell::Haste,
                Spell::Teleport,
                Spell::RaiseTheDead,
                Spell::BattleHymn,
                Spell::HealingPlume,
                Spell::FogCloud,
                Spell::BerserkerRage,
            ],
            SpellCategory::Utility => &[
                Spell::Telekinesis,
                Spell::PhantasmalForce,
                Spell::Banishment,
                Spell::ArcaneCrystal,
            ],
        }
    }
}

impl Spell {
    /// Returns all available spells in order.
    pub const fn all() -> &'static [Spell] {
        &[
            Spell::MagicMissile,
            Spell::Disintegrate,
            Spell::Fireball,
            Spell::GuardianCircle,
            Spell::ChainLightning,
            Spell::FingerOfDeath,
            Spell::RaiseTheDead,
            Spell::Teleport,
            Spell::WallOfStone,
            Spell::BlackHole,
            Spell::Squall,
            Spell::WallOfFire,
            Spell::Entangle,
            Spell::Haste,
            Spell::SpikeGrowth,
            Spell::LightningRod,
            Spell::Telekinesis,
            Spell::MeteorFall,
            Spell::MarkOfDeath,
            Spell::PlagueWind,
            Spell::HypnoticPattern,
            Spell::Sleep,
            Spell::Grease,
            Spell::FogCloud,
            Spell::BattleHymn,
            Spell::HealingPlume,
            Spell::BerserkerRage,
            Spell::PhantasmalForce,
            Spell::Banishment,
            Spell::Polymorph,
            Spell::ArcaneCrystal,
        ]
    }

    /// Returns the category this spell belongs to.
    #[allow(dead_code)]
    pub const fn category(&self) -> SpellCategory {
        match self {
            Spell::MagicMissile
            | Spell::Disintegrate
            | Spell::Fireball
            | Spell::ChainLightning
            | Spell::FingerOfDeath
            | Spell::LightningRod
            | Spell::MeteorFall
            | Spell::MarkOfDeath
            | Spell::PlagueWind => SpellCategory::Offense,

            Spell::BlackHole
            | Spell::WallOfStone
            | Spell::WallOfFire
            | Spell::Entangle
            | Spell::SpikeGrowth
            | Spell::Squall
            | Spell::HypnoticPattern
            | Spell::Sleep
            | Spell::Grease
            | Spell::Polymorph => SpellCategory::Control,

            Spell::GuardianCircle
            | Spell::Haste
            | Spell::Teleport
            | Spell::RaiseTheDead
            | Spell::BattleHymn
            | Spell::HealingPlume
            | Spell::FogCloud
            | Spell::BerserkerRage => SpellCategory::Support,

            Spell::Telekinesis
            | Spell::PhantasmalForce
            | Spell::Banishment
            | Spell::ArcaneCrystal => SpellCategory::Utility,
        }
    }

    /// Returns a single-line display name for UI contexts (no newlines).
    pub const fn display_name(&self) -> &'static str {
        match self {
            Spell::MagicMissile => "Magic Missile",
            Spell::Disintegrate => "Disintegrate",
            Spell::Fireball => "Fireball",
            Spell::GuardianCircle => "Guardian Circle",
            Spell::ChainLightning => "Chain Lightning",
            Spell::FingerOfDeath => "Finger of Death",
            Spell::RaiseTheDead => "Raise The Dead",
            Spell::Teleport => "Teleport",
            Spell::WallOfStone => "Wall of Stone",
            Spell::BlackHole => "Black Hole",
            Spell::Squall => "Squall",
            Spell::WallOfFire => "Wall of Fire",
            Spell::Entangle => "Entangle",
            Spell::Haste => "Haste",
            Spell::SpikeGrowth => "Spike Growth",
            Spell::LightningRod => "Lightning Rod",
            Spell::Telekinesis => "Telekinesis",
            Spell::MeteorFall => "Meteor Fall",
            Spell::MarkOfDeath => "Mark of Death",
            Spell::PlagueWind => "Plague Wind",
            Spell::HypnoticPattern => "Hypnotic Pattern",
            Spell::Sleep => "Sleep",
            Spell::Grease => "Grease",
            Spell::FogCloud => "Fog Cloud",
            Spell::BattleHymn => "Battle Hymn",
            Spell::HealingPlume => "Healing Plume",
            Spell::BerserkerRage => "Berserker Rage",
            Spell::PhantasmalForce => "Phantasmal Force",
            Spell::Banishment => "Banishment",
            Spell::Polymorph => "Polymorph",
            Spell::ArcaneCrystal => "Arcane Crystal",
        }
    }

    /// Returns the display name for this spell with newlines between words.
    pub const fn name(&self) -> &'static str {
        match self {
            Spell::MagicMissile => "Magic\nMissile",
            Spell::Disintegrate => "Disintegrate",
            Spell::Fireball => "Fireball",
            Spell::GuardianCircle => "Guardian\nCircle",
            Spell::ChainLightning => "Chain\nLightning",
            Spell::FingerOfDeath => "Finger\nof\nDeath",
            Spell::RaiseTheDead => "Raise\nThe\nDead",
            Spell::Teleport => "Teleport",
            Spell::WallOfStone => "Wall of\nStone",
            Spell::BlackHole => "Black\nHole",
            Spell::Squall => "Squall",
            Spell::WallOfFire => "Wall of\nFire",
            Spell::Entangle => "Entangle",
            Spell::Haste => "Haste",
            Spell::SpikeGrowth => "Spike\nGrowth",
            Spell::LightningRod => "Lightning\nRod",
            Spell::Telekinesis => "Telekinesis",
            Spell::MeteorFall => "Meteor\nFall",
            Spell::MarkOfDeath => "Mark of\nDeath",
            Spell::PlagueWind => "Plague\nWind",
            Spell::HypnoticPattern => "Hypnotic\nPattern",
            Spell::Sleep => "Sleep",
            Spell::Grease => "Grease",
            Spell::FogCloud => "Fog\nCloud",
            Spell::BattleHymn => "Battle\nHymn",
            Spell::HealingPlume => "Healing\nPlume",
            Spell::BerserkerRage => "Berserker\nRage",
            Spell::PhantasmalForce => "Phantasmal\nForce",
            Spell::Banishment => "Banishment",
            Spell::Polymorph => "Polymorph",
            Spell::ArcaneCrystal => "Arcane\nCrystal",
        }
    }

    /// Returns the description for this spell.
    pub const fn description(&self) -> &'static str {
        match self {
            Spell::MagicMissile => {
                "Fires homing missiles that seek nearby units. Channels faster over time."
            }
            Spell::Disintegrate => {
                "Projects a beam toward the cursor, dealing continuous damage to units in its path."
            }
            Spell::Fireball => {
                "Launches an explosive fireball at the cursor that deals area damage on impact."
            }
            Spell::GuardianCircle => {
                "Creates a protective circle at the cursor that gives units extra temporary health."
            }
            Spell::ChainLightning => {
                "Strikes the nearest unit with lightning that splits and spreads to nearby targets."
            }
            Spell::FingerOfDeath => {
                "Fires a deadly beam at the cursor, dealing heavy damage to units in its path."
            }
            Spell::RaiseTheDead => "Resurrects corpses near the cursor.",
            Spell::Teleport => "Teleports all units near the cursor to a chosen destination.",
            Spell::WallOfStone => {
                "Drag to raise an impassable stone wall that blocks all movement and projectiles for 20 seconds."
            }
            Spell::BlackHole => {
                "Creates a gravitational sphere that pulls units inward in a spiral. Lasts 10 seconds, growing in size and strength over time."
            }
            Spell::Squall => {
                "Summons a storm that rains ice down on a targeted area, dealing frost damage and slowing enemies. Requires concentration to maintain."
            }
            Spell::WallOfFire => {
                "Drag a line of fire that burns all units walking through it. Lasts 36 seconds."
            }
            Spell::Entangle => {
                "Roots all units in radius, preventing movement but not attacks. Lasts 5 seconds."
            }
            Spell::Haste => "Grants all units in radius increased movement speed for 10 seconds.",
            Spell::SpikeGrowth => {
                "Creates a persistent zone that damages and slows all units inside. Lasts 15 seconds."
            }
            Spell::LightningRod => {
                "Places a lightning rod that periodically attracts lightning strikes. Arcs of electricity jump to all nearby units, friend or foe."
            }
            Spell::Telekinesis => {
                "Retrieves ingredients dropped by fallen enemies. Click near a drop to pull it to you."
            }
            Spell::MeteorFall => {
                "Rains meteors from the sky, each exploding on impact and leaving persistent burning ground. Requires concentration to maintain."
            }
            Spell::MarkOfDeath => {
                "Marks a single enemy for death. The marked unit takes heavily increased damage from all sources for 8 seconds."
            }
            Spell::PlagueWind => {
                "Summons a toxic cloud that drifts across the battlefield, poisoning all units it passes through."
            }
            Spell::HypnoticPattern => {
                "Mesmerizes all units in radius, preventing movement and attacks. Taking any damage breaks the effect."
            }
            Spell::Sleep => {
                "Puts all units in radius to sleep. Sleeping units take bonus damage from the first hit that wakes them."
            }
            Spell::Grease => {
                "Covers the ground in slippery oil that slows units. Can be ignited by fire spells for a devastating explosion."
            }
            Spell::FogCloud => {
                "Blankets an area in supernatural fog. Units inside have a chance to evade incoming attacks. Lasts 12 seconds."
            }
            Spell::BattleHymn => {
                "Chants a magical war hymn that increases damage and attack speed for all units in radius."
            }
            Spell::HealingPlume => {
                "Creates a healing zone where all units slowly regenerate health over time. Lasts 10 seconds."
            }
            Spell::BerserkerRage => {
                "Enrages all units in radius. They deal much more damage but also take more damage. Lasts 8 seconds."
            }
            Spell::PhantasmalForce => {
                "Conjures illusory defenders that draw enemy attention. Illusions have very low health and deal no damage."
            }
            Spell::Banishment => {
                "Banishes a single enemy from the battlefield for 8 seconds. The unit vanishes and returns when the effect ends."
            }
            Spell::Polymorph => {
                "Transforms an enemy into a harmless sheep for 10 seconds. Kill the sheep for a permanent kill, or it reverts."
            }
            Spell::ArcaneCrystal => {
                "Places a magical crystal that amplifies your spells. Hit it with projectiles, beams, or meteors to scatter smaller copies at nearby enemies."
            }
        }
    }

    /// Returns flavor text shown on the progress screen when the spell is locked.
    pub const fn locked_description(&self) -> &'static str {
        match self {
            Spell::MagicMissile => "The first spell every wizard learns. Usually by accident.",
            Spell::Disintegrate => "Point. Shoot. Nothing remains.",
            Spell::Fireball => "The answer to every wizard's problems. And sometimes the cause.",
            Spell::GuardianCircle => "Stand in the glowy circle. It's not that complicated.",
            Spell::ChainLightning => "Electricity doesn't care about your feelings.",
            Spell::FingerOfDeath => {
                "Technically it's the whole hand, but that sounds less dramatic."
            }
            Spell::RaiseTheDead => "They were just resting. Aggressively.",
            Spell::Teleport => "Be somewhere else. Immediately.",
            Spell::WallOfStone => "The ultimate 'you shall not pass' moment.",
            Spell::BlackHole => "What goes in doesn't come out. Wizard's promise.",
            Spell::Squall => "Nothing says 'tactical advantage' like a localized ice storm.",
            Spell::WallOfFire => "Fire doesn't pick sides. Neither should you.",
            Spell::Entangle => "Vines don't ask whose feet they're grabbing.",
            Spell::Haste => "Fast friends, fast enemies. Same spell.",
            Spell::SpikeGrowth => "Nature reclaims the battlefield. Everyone suffers equally.",
            Spell::LightningRod => {
                "Stand near the tall metal thing during a storm. What could go wrong?"
            }
            Spell::Telekinesis => "Mind over matter. Literally.",
            Spell::MeteorFall => "When one fireball just isn't enough.",
            Spell::MarkOfDeath => "Everyone hits harder when they know where to aim.",
            Spell::PlagueWind => "A gentle breeze. With consequences.",
            Spell::HypnoticPattern => "Ooh, pretty lights. Why can't I move?",
            Spell::Sleep => "Shh. They're having the best nap of their lives.",
            Spell::Grease => "What's worse than slipping? Slipping while on fire.",
            Spell::FogCloud => "Can't hit what you can't see.",
            Spell::BattleHymn => "Nothing motivates like a good war song.",
            Spell::HealingPlume => "Stand in the glowy green zone. Trust the wizard.",
            Spell::BerserkerRage => "Hit harder. Get hit harder. Worth it.",
            Spell::PhantasmalForce => "Are those real soldiers? Only one way to find out.",
            Spell::Banishment => "Poof. Gone. Back in a minute.",
            Spell::Polymorph => "Baa.",
            Spell::ArcaneCrystal => {
                "A prism for the ambitious. What goes in comes out... everywhere."
            }
        }
    }

    /// Returns the control instructions for this spell.
    pub const fn instructions(&self) -> &'static str {
        match self {
            Spell::MagicMissile => "Click and hold to channel",
            Spell::Disintegrate => "Click and hold to channel",
            Spell::Fireball => "Click and hold to cast",
            Spell::GuardianCircle => "Click and hold to place",
            Spell::ChainLightning => "Click and hold to cast",
            Spell::FingerOfDeath => "Click and hold to cast",
            Spell::RaiseTheDead => "Click and hold to channel",
            Spell::Teleport => "Click to place destination, then click and hold to cast",
            Spell::WallOfStone => "Click and drag to place wall",
            Spell::BlackHole => "Click and hold to cast",
            Spell::Squall => "Click and hold to place storm",
            Spell::WallOfFire => "Click and drag to place fire wall",
            Spell::Entangle => "Click and hold to place",
            Spell::Haste => "Click and hold to place",
            Spell::SpikeGrowth => "Click and hold to place",
            Spell::LightningRod => "Click and hold to place",
            Spell::Telekinesis => "Click near a drop to retrieve",
            Spell::MeteorFall => "Click and hold to place meteor storm",
            Spell::MarkOfDeath => "Click and hold to cast",
            Spell::PlagueWind => "Click and hold to place",
            Spell::HypnoticPattern => "Click and hold to place",
            Spell::Sleep => "Click and hold to place",
            Spell::Grease => "Click and hold to place",
            Spell::FogCloud => "Click and hold to place",
            Spell::BattleHymn => "Click and hold to place",
            Spell::HealingPlume => "Click and hold to place",
            Spell::BerserkerRage => "Click and hold to place",
            Spell::PhantasmalForce => "Click and hold to place",
            Spell::Banishment => "Click and hold to cast",
            Spell::Polymorph => "Click and hold to cast",
            Spell::ArcaneCrystal => "Click and hold to place",
        }
    }

    /// Returns the PrimedSpell configuration for this spell.
    pub const fn primed_config(self) -> PrimedSpell {
        use crate::game::units::wizard::spells::{
            arcane_crystal_constants, banishment_constants, battle_hymn_constants,
            berserker_rage_constants, black_hole_constants, chain_lightning_constants,
            disintegrate_constants, entangle_constants, finger_of_death_constants,
            fireball_constants, fog_cloud_constants, grease_constants, guardian_circle_constants,
            haste_constants, healing_plume_constants, hypnotic_pattern_constants,
            lightning_rod_constants, magic_missile_constants, mark_of_death_constants,
            meteor_fall_constants, phantasmal_force_constants, plague_wind_constants,
            polymorph_constants, raise_the_dead_constants, sleep_constants, spike_growth_constants,
            squall_constants, telekinesis_constants, teleport_constants, wall_of_fire_constants,
            wall_of_stone_constants,
        };

        match self {
            Spell::MagicMissile => magic_missile_constants::PRIMED_MAGIC_MISSILE,
            Spell::Disintegrate => disintegrate_constants::PRIMED_DISINTEGRATE,
            Spell::Fireball => fireball_constants::PRIMED_FIREBALL,
            Spell::GuardianCircle => guardian_circle_constants::PRIMED_GUARDIAN_CIRCLE,
            Spell::ChainLightning => chain_lightning_constants::PRIMED_CHAIN_LIGHTNING,
            Spell::FingerOfDeath => finger_of_death_constants::PRIMED_FINGER_OF_DEATH,
            Spell::RaiseTheDead => raise_the_dead_constants::PRIMED_RAISE_THE_DEAD,
            Spell::Teleport => teleport_constants::PRIMED_TELEPORT,
            Spell::WallOfStone => wall_of_stone_constants::PRIMED_WALL_OF_STONE,
            Spell::BlackHole => black_hole_constants::PRIMED_BLACK_HOLE,
            Spell::Squall => squall_constants::PRIMED_SQUALL,
            Spell::WallOfFire => wall_of_fire_constants::PRIMED_WALL_OF_FIRE,
            Spell::Entangle => entangle_constants::PRIMED_ENTANGLE,
            Spell::Haste => haste_constants::PRIMED_HASTE,
            Spell::SpikeGrowth => spike_growth_constants::PRIMED_SPIKE_GROWTH,
            Spell::LightningRod => lightning_rod_constants::PRIMED_LIGHTNING_ROD,
            Spell::Telekinesis => telekinesis_constants::PRIMED_TELEKINESIS,
            Spell::MeteorFall => meteor_fall_constants::PRIMED_METEOR_FALL,
            Spell::MarkOfDeath => mark_of_death_constants::PRIMED_MARK_OF_DEATH,
            Spell::PlagueWind => plague_wind_constants::PRIMED_PLAGUE_WIND,
            Spell::HypnoticPattern => hypnotic_pattern_constants::PRIMED_HYPNOTIC_PATTERN,
            Spell::Sleep => sleep_constants::PRIMED_SLEEP,
            Spell::Grease => grease_constants::PRIMED_GREASE,
            Spell::FogCloud => fog_cloud_constants::PRIMED_FOG_CLOUD,
            Spell::BattleHymn => battle_hymn_constants::PRIMED_BATTLE_HYMN,
            Spell::HealingPlume => healing_plume_constants::PRIMED_HEALING_PLUME,
            Spell::BerserkerRage => berserker_rage_constants::PRIMED_BERSERKER_RAGE,
            Spell::PhantasmalForce => phantasmal_force_constants::PRIMED_PHANTASMAL_FORCE,
            Spell::Banishment => banishment_constants::PRIMED_BANISHMENT,
            Spell::Polymorph => polymorph_constants::PRIMED_POLYMORPH,
            Spell::ArcaneCrystal => arcane_crystal_constants::PRIMED_ARCANE_CRYSTAL,
        }
    }

    /// Returns the type of damage this spell deals.
    pub const fn damage_type(&self) -> DamageType {
        match self {
            Spell::MagicMissile => DamageType::Force,
            Spell::Disintegrate => DamageType::Fire,
            Spell::Fireball => DamageType::Fire,
            Spell::ChainLightning => DamageType::Electric,
            Spell::BlackHole => DamageType::Force,
            Spell::GuardianCircle => DamageType::Force,
            Spell::FingerOfDeath => DamageType::Necrotic,
            Spell::RaiseTheDead => DamageType::Necrotic,
            Spell::Teleport => DamageType::Force,
            Spell::WallOfStone => DamageType::Force,
            Spell::Squall => DamageType::Frost,
            Spell::WallOfFire => DamageType::Fire,
            Spell::Entangle => DamageType::Nature,
            Spell::Haste => DamageType::Force,
            Spell::SpikeGrowth => DamageType::Nature,
            Spell::LightningRod => DamageType::Electric,
            Spell::Telekinesis => DamageType::Force,
            Spell::MeteorFall => DamageType::Fire,
            Spell::MarkOfDeath => DamageType::Necrotic,
            Spell::PlagueWind => DamageType::Necrotic,
            Spell::HypnoticPattern => DamageType::Force,
            Spell::Sleep => DamageType::Force,
            Spell::Grease => DamageType::Nature,
            Spell::FogCloud => DamageType::Frost,
            Spell::BattleHymn => DamageType::Force,
            Spell::HealingPlume => DamageType::Nature,
            Spell::BerserkerRage => DamageType::Force,
            Spell::PhantasmalForce => DamageType::Force,
            Spell::Banishment => DamageType::Force,
            Spell::Polymorph => DamageType::Nature,
            Spell::ArcaneCrystal => DamageType::Force,
        }
    }

    /// Returns the Arcane Insight cost to research this spell.
    /// Default spells (MagicMissile, Telekinesis) have cost 0 — they start unlocked.
    pub const fn research_cost(&self) -> u32 {
        match self {
            Spell::MagicMissile | Spell::Telekinesis => 0,
            // Root spells (immediately researchable)
            Spell::Disintegrate => 30,
            Spell::Grease => 30,
            Spell::ChainLightning => 30,
            Spell::FingerOfDeath => 30,
            Spell::GuardianCircle => 30,
            Spell::WallOfStone => 30,
            // Second-tier (requires predecessor)
            Spell::Fireball => 50,
            Spell::Entangle => 50,
            Spell::MarkOfDeath => 50,
            Spell::HealingPlume => 50,
            Spell::BerserkerRage => 50,
            Spell::SpikeGrowth => 60,
            Spell::LightningRod => 60,
            Spell::RaiseTheDead => 60,
            Spell::Haste => 60,
            Spell::Teleport => 60,
            // Third-tier
            Spell::WallOfFire => 80,
            Spell::BattleHymn => 80,
            Spell::HypnoticPattern => 80,
            Spell::PlagueWind => 80,
            // Fourth-tier (capstones)
            Spell::MeteorFall => 100,
            // Misc (require N total spells researched)
            Spell::Squall => 70,
            Spell::PhantasmalForce => 50,
            Spell::FogCloud => 60,
            Spell::Sleep => 80,
            Spell::Banishment => 90,
            Spell::BlackHole => 120,
            Spell::Polymorph => 100,
            Spell::ArcaneCrystal => 80,
        }
    }

    /// Returns the prerequisite spell that must be researched first, if any.
    pub const fn prerequisite(&self) -> Option<Spell> {
        match self {
            // Fire chain: Disintegrate → Fireball → Wall of Fire → Meteor Fall
            Spell::Fireball => Some(Spell::Disintegrate),
            Spell::WallOfFire => Some(Spell::Fireball),
            Spell::MeteorFall => Some(Spell::WallOfFire),
            // Nature chain: Grease → Entangle → Spike Growth, Entangle → Healing Plume
            Spell::Entangle => Some(Spell::Grease),
            Spell::HealingPlume => Some(Spell::Entangle),
            Spell::SpikeGrowth => Some(Spell::Entangle),
            // Electric chain: Chain Lightning → Lightning Rod
            Spell::LightningRod => Some(Spell::ChainLightning),
            // Necrotic chain: Finger of Death → Raise the Dead, → Mark of Death → Plague Wind
            Spell::RaiseTheDead => Some(Spell::FingerOfDeath),
            Spell::MarkOfDeath => Some(Spell::FingerOfDeath),
            Spell::PlagueWind => Some(Spell::MarkOfDeath),
            // Force chains: Guardian Circle → Haste → Battle Hymn, Guardian Circle → Berserker Rage
            Spell::Haste => Some(Spell::GuardianCircle),
            Spell::BerserkerRage => Some(Spell::GuardianCircle),
            Spell::BattleHymn => Some(Spell::Haste),
            // Force chains: Wall of Stone → Teleport → Hypnotic Pattern
            Spell::Teleport => Some(Spell::WallOfStone),
            Spell::HypnoticPattern => Some(Spell::Teleport),
            _ => None,
        }
    }

    /// Returns the minimum number of total spells that must be researched
    /// before this spell becomes available (0 = no requirement).
    /// Used for miscellaneous spells not in a prerequisite chain.
    pub const fn required_total_spells(&self) -> u32 {
        match self {
            Spell::Squall => 4,
            Spell::PhantasmalForce => 5,
            Spell::FogCloud => 5,
            Spell::Sleep => 6,
            Spell::Banishment => 7,
            Spell::BlackHole => 8,
            Spell::Polymorph => 8,
            Spell::ArcaneCrystal => 6,
            _ => 0,
        }
    }

    /// Returns true if this spell transitions to channeling after the cast completes.
    ///
    /// Channeled spells keep CastingState::Channeling while the mouse is held,
    /// producing repeated effects over time.
    pub const fn is_channeled(&self) -> bool {
        matches!(
            self,
            Spell::MagicMissile | Spell::Disintegrate | Spell::RaiseTheDead
        )
    }

    /// Returns true if this spell is researchable (not a default spell).
    pub const fn is_researchable(&self) -> bool {
        self.research_cost() > 0
    }

    /// Returns all researchable spells (excludes MagicMissile and Telekinesis).
    pub fn researchable() -> Vec<Spell> {
        Spell::all()
            .iter()
            .copied()
            .filter(|s| s.is_researchable())
            .collect()
    }
}

/// Component tracking which spell is currently primed for casting.
///
/// Contains both the spell type and its associated properties like cast time.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct PrimedSpell {
    pub spell: Spell,
    /// Time required to cast this spell before it activates (in seconds).
    pub cast_time: f32,
    /// Empowerment multiplier for spell effectiveness (1.0 = normal, 1.25 = 25% bonus, etc.).
    pub empowerment: f32,
    /// Whether the empowerment has been consumed by starting a cast.
    pub empowerment_consumed: bool,
    /// Mana cost multiplier (1.0 = normal, 0.5 = half cost, 2.0 = double cost).
    pub mana_multiplier: f32,
    /// Range multiplier for spell radius and distance (1.0 = normal, 1.5 = 50% more range).
    pub range_multiplier: f32,
}

impl PrimedSpell {
    /// Creates an empowered version of this primed spell with the given multiplier.
    pub const fn with_empowerment(mut self, multiplier: f32) -> Self {
        self.empowerment = multiplier;
        self.cast_time /= multiplier;
        self.empowerment_consumed = false;
        self
    }

    /// Applies empowerment scaling to a value.
    pub fn scale(&self, value: f32) -> f32 {
        value * self.empowerment
    }

    /// Marks the empowerment as consumed (called when starting a cast).
    pub fn consume_empowerment(&mut self) {
        self.empowerment_consumed = true;
    }

    /// Returns true if empowerment needs to be reset.
    pub fn should_reset_empowerment(&self) -> bool {
        self.empowerment != 1.0 && self.empowerment_consumed
    }

    /// Resets empowerment to 1.0 and restores original cast time.
    pub fn reset_empowerment(&mut self) {
        self.empowerment = 1.0;
        self.cast_time = self.spell.primed_config().cast_time;
        self.empowerment_consumed = false;
    }
}

/// Wizard component with spell casting stats.
///
/// These are the wizard's actual effective stats after all modifiers are applied.
/// Various systems (archetypes, cauldron buffs, etc.) modify these values.
#[derive(Component)]
pub struct Wizard {
    /// Maximum distance from wizard position that spells can be cast (in units).
    pub spell_range: f32,
    /// Multiplier for spell mana costs (1.0 = normal, 0.5 = half cost, 2.0 = double cost).
    pub mana_cost_multiplier: f32,
    /// Multiplier for spell power/damage (1.0 = normal, 1.5 = 50% more damage).
    pub spell_power_multiplier: f32,
    /// Multiplier for cast speed (1.0 = normal, 2.0 = twice as fast).
    pub cast_speed_multiplier: f32,
}

impl Wizard {
    /// Creates a new Wizard with the given spell range and default multipliers.
    pub const fn new(spell_range: f32) -> Self {
        Self {
            spell_range,
            mana_cost_multiplier: 1.0,
            spell_power_multiplier: 1.0,
            cast_speed_multiplier: 1.0,
        }
    }
}

/// Mana component for the wizard.
///
/// Tracks current and maximum mana for spell casting.
#[derive(Component)]
pub struct Mana {
    /// Current mana amount.
    pub current: f32,
    /// Maximum mana capacity.
    pub max: f32,
}

impl Mana {
    /// Creates a new Mana component with the given maximum.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Returns true if there is enough mana for the cost.
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= cost
    }

    /// Consumes mana, returning true if successful.
    pub fn consume(&mut self, cost: f32) -> bool {
        if self.can_afford(cost) {
            self.current -= cost;
            true
        } else {
            false
        }
    }

    /// Regenerates mana, clamped to max.
    pub fn regenerate(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Returns mana as a percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

/// Mana regeneration component.
///
/// Defines how fast mana regenerates per second.
#[derive(Component)]
pub struct ManaRegen {
    /// Mana regenerated per second.
    pub rate: f32,
}

impl ManaRegen {
    /// Creates a new ManaRegen component.
    pub const fn new(rate: f32) -> Self {
        Self { rate }
    }
}

/// Casting state component for the wizard.
///
/// Tracks active spell casting progress and channeling.
#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub enum CastingState {
    /// Not casting or channeling.
    #[default]
    Resting,
    /// Currently casting a spell.
    Casting {
        /// Time accumulated toward cast completion (in seconds).
        elapsed: f32,
    },
    /// Channeling after cast completion.
    Channeling {
        /// Total time spent channeling (in seconds).
        total_time: f32,
        /// Time since last channeled spell effect (in seconds).
        time_since_last_effect: f32,
    },
}

impl CastingState {
    /// Creates a new CastingState in the Resting state.
    pub const fn new() -> Self {
        Self::Resting
    }

    /// Starts a new cast.
    pub fn start_cast(&mut self) {
        *self = Self::Casting { elapsed: 0.0 };
    }

    /// Transitions from casting to channeling.
    pub fn start_channeling(&mut self) {
        *self = Self::Channeling {
            total_time: 0.0,
            time_since_last_effect: 0.0,
        };
    }

    /// Cancels the current cast or channel, returning to Resting.
    pub fn cancel(&mut self) {
        *self = Self::Resting;
    }

    /// Advances the cast by the given time (only during Casting state).
    pub fn advance(&mut self, delta: f32) {
        if let Self::Casting { elapsed } = self {
            *elapsed += delta;
        }
    }

    /// Advances channeling timers (only during Channeling state).
    pub fn advance_channel(&mut self, delta: f32) {
        if let Self::Channeling {
            total_time,
            time_since_last_effect,
        } = self
        {
            *total_time += delta;
            *time_since_last_effect += delta;
        }
    }

    /// Resets the time since last channel effect (call when spawning a channeled spell).
    pub fn reset_channel_interval(&mut self) {
        if let Self::Channeling {
            time_since_last_effect,
            ..
        } = self
        {
            *time_since_last_effect = 0.0;
        }
    }

    /// Returns the current channel interval based on how long channeling has been active.
    ///
    /// Starts at `initial_interval` and decreases to `min_interval` over `ramp_time`.
    pub fn channel_interval(
        &self,
        initial_interval: f32,
        min_interval: f32,
        ramp_time: f32,
    ) -> f32 {
        if let Self::Channeling { total_time, .. } = self {
            if ramp_time <= 0.0 {
                return min_interval;
            }

            let t = (total_time / ramp_time).min(1.0);
            initial_interval + (min_interval - initial_interval) * t
        } else {
            initial_interval
        }
    }

    /// Returns true if enough time has passed to spawn another channeled spell.
    pub fn should_channel(&self, initial_interval: f32, min_interval: f32, ramp_time: f32) -> bool {
        if let Self::Channeling {
            time_since_last_effect,
            ..
        } = self
        {
            *time_since_last_effect
                >= self.channel_interval(initial_interval, min_interval, ramp_time)
        } else {
            false
        }
    }

    /// Returns true if the cast is complete (ready to transition to channeling).
    pub fn is_complete(&self, cast_time: f32) -> bool {
        if let Self::Casting { elapsed } = self {
            *elapsed >= cast_time
        } else {
            false
        }
    }

    /// Returns cast progress as a percentage (0.0 to 1.0).
    /// Returns 1.0 when channeling to keep bar full.
    pub fn progress(&self, cast_time: f32) -> f32 {
        match self {
            Self::Resting => 0.0,
            Self::Casting { elapsed } => {
                if cast_time > 0.0 {
                    (elapsed / cast_time).min(1.0)
                } else {
                    0.0
                }
            }
            Self::Channeling { .. } => 1.0,
        }
    }
}

/// Marker component to track when the wizard is actively casting a spell.
///
/// This marker is added when a spell cast begins and removed immediately after
/// the spell completes. It prevents immediate recast while the mouse button is
/// held down, but allows immediate casting on the next mouse press without
/// requiring a double-click.
///
/// The marker stores an optional entity (like a circle indicator) that should
/// be despawned when the cast is cancelled or completed.
#[derive(Component)]
pub struct SpellCaster {
    /// Optional entity to despawn when cast is cancelled/completed (e.g., circle indicator).
    pub indicator_entity: Option<Entity>,
}

impl SpellCaster {
    /// Creates a new SpellCaster marker with no indicator.
    pub const fn new() -> Self {
        Self {
            indicator_entity: None,
        }
    }

    /// Creates a new SpellCaster marker with an indicator entity.
    pub const fn with_indicator(indicator_entity: Entity) -> Self {
        Self {
            indicator_entity: Some(indicator_entity),
        }
    }
}

/// Marker for the locally-controlled wizard.
///
/// In single-player, the one wizard gets this marker.
/// In multiplayer, each peer's own wizard gets this marker.
/// Spell casting systems query `With<LocalWizard>` instead of `With<Wizard>`
/// to ensure `single()` works with two wizard entities present.
#[derive(Component)]
pub struct LocalWizard;

/// Tracks the wizard sprite sheet animation state.
#[derive(Component)]
pub struct WizardAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
}

impl WizardAnimation {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
        }
    }

    /// Advances the animation timer and returns true if frame changed.
    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= super::constants::WIZARD_FRAME_DURATION {
            self.elapsed -= super::constants::WIZARD_FRAME_DURATION;
            self.current_frame =
                (self.current_frame + 1) % super::constants::WIZARD_SPRITE_FRAMES;
            true
        } else {
            false
        }
    }

    /// Calculates UV offset for current frame in a 3x3 grid.
    pub fn uv_offset(&self) -> (f32, f32) {
        let grid_size = super::constants::WIZARD_SPRITE_GRID_SIZE as f32;
        let frame_size = 1.0 / grid_size;

        let row = (self.current_frame / super::constants::WIZARD_SPRITE_GRID_SIZE) as f32;
        let col = (self.current_frame % super::constants::WIZARD_SPRITE_GRID_SIZE) as f32;

        (col * frame_size, row * frame_size)
    }
}

/// Stores the wizard sprite sheet texture handle.
#[derive(Resource)]
pub struct WizardAssets {
    pub sprite_texture: Handle<Image>,
}

/// Marker for the guest's wizard entity on the host.
///
/// Only added in multiplayer on the host. The host processes incoming
/// `SpellCommand` messages and drives this wizard's casting state.
#[derive(Component)]
pub struct GuestWizard;

/// Abstract wizard input for spell casting — same shape regardless of source.
///
/// Local wizard builds this from mouse state + camera raycast.
/// Guest wizard builds this from `GuestInputState` + `GuestCursorPosition`.
/// Spell casting logic consumes this without knowing the input source.
pub struct WizardInput {
    /// True on the frame the cast button was first pressed.
    pub just_pressed: bool,
    /// True while the cast button is held.
    pub pressed: bool,
    /// True on the frame the cast button was released.
    pub just_released: bool,
    /// Cursor world position on the battlefield (Y=0 plane).
    pub cursor_pos: Option<Vec3>,
}
