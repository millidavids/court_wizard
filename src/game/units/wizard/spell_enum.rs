use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::spell_status_effects::StatusEffectKind;
use super::wizard_state::PrimedSpell;
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
    Sleep,
    Grease,
    FogCloud,
    BattleHymn,
    HealingPlume,
    BerserkerRage,
    Banishment,
    Polymorph,
    ArcaneCrystal,
    Dispel,
    MindControl,
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
                Spell::Sleep,
                Spell::Grease,
                Spell::Polymorph,
                Spell::MindControl,
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
                Spell::Banishment,
                Spell::ArcaneCrystal,
                Spell::Dispel,
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
            Spell::Sleep,
            Spell::Grease,
            Spell::FogCloud,
            Spell::BattleHymn,
            Spell::HealingPlume,
            Spell::BerserkerRage,
            Spell::Banishment,
            Spell::Polymorph,
            Spell::ArcaneCrystal,
            Spell::Dispel,
            Spell::MindControl,
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
            | Spell::Sleep
            | Spell::Grease
            | Spell::Polymorph
            | Spell::MindControl => SpellCategory::Control,

            Spell::GuardianCircle
            | Spell::Haste
            | Spell::Teleport
            | Spell::RaiseTheDead
            | Spell::BattleHymn
            | Spell::HealingPlume
            | Spell::FogCloud
            | Spell::BerserkerRage => SpellCategory::Support,

            Spell::Telekinesis | Spell::Banishment | Spell::ArcaneCrystal | Spell::Dispel => {
                SpellCategory::Utility
            }
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
            Spell::WallOfStone => "Wall of Dirt",
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
            Spell::Sleep => "Sleep",
            Spell::Grease => "Grease",
            Spell::FogCloud => "Fog Cloud",
            Spell::BattleHymn => "Battle Hymn",
            Spell::HealingPlume => "Healing Plume",
            Spell::BerserkerRage => "Berserker Rage",
            Spell::Banishment => "Banishment",
            Spell::Polymorph => "Polymorph",
            Spell::ArcaneCrystal => "Arcane Crystal",
            Spell::Dispel => "Dispel",
            Spell::MindControl => "Mind Control",
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
            Spell::WallOfStone => "Wall of\nDirt",
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
            Spell::Sleep => "Sleep",
            Spell::Grease => "Grease",
            Spell::FogCloud => "Fog\nCloud",
            Spell::BattleHymn => "Battle\nHymn",
            Spell::HealingPlume => "Healing\nPlume",
            Spell::BerserkerRage => "Berserker\nRage",
            Spell::Banishment => "Banishment",
            Spell::Polymorph => "Polymorph",
            Spell::ArcaneCrystal => "Arcane\nCrystal",
            Spell::Dispel => "Dispel",
            Spell::MindControl => "Mind\nControl",
        }
    }

    /// Returns the description for this spell.
    pub const fn description(&self) -> &'static str {
        match self {
            Spell::MagicMissile => "Fires a volley of 3 homing missiles that seek nearby units.",
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
                "Drag to raise an impassable dirt wall that blocks all movement and projectiles for 20 seconds."
            }
            Spell::BlackHole => {
                "Creates a gravitational sphere that pulls units inward in a spiral. Lasts 10 seconds, growing in size and strength over time."
            }
            Spell::Squall => {
                "Summons a storm that rains ice down on a targeted area, dealing frost damage and slowing enemies. Concentration spell — reserves mana while active."
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
                "Rains meteors from the sky, each exploding on impact and leaving persistent burning ground. Concentration spell — reserves mana while active."
            }
            Spell::MarkOfDeath => {
                "Marks a single enemy for death. The marked unit takes heavily increased damage from all sources for 8 seconds."
            }
            Spell::PlagueWind => {
                "Summons a toxic cloud that drifts across the battlefield, poisoning all units it passes through."
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
            Spell::Banishment => {
                "Banishes a single enemy from the battlefield for 8 seconds. The unit vanishes and returns when the effect ends."
            }
            Spell::Polymorph => {
                "Transforms an enemy into a harmless sheep for 10 seconds. Kill the sheep for a permanent kill, or it reverts."
            }
            Spell::ArcaneCrystal => {
                "Places a magical crystal that amplifies your spells. Hit it with projectiles, beams, or meteors to scatter smaller copies at nearby enemies."
            }
            Spell::Dispel => {
                "Fires a bolt of nullifying energy at the cursor. On impact, an expanding wave removes any spell effects it touches."
            }
            Spell::MindControl => {
                "Hold to cast on the nearest enemy to your cursor. The target turns against its allies, attacking them instead. Wears off over time."
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
            Spell::Sleep => "Shh. They're having the best nap of their lives.",
            Spell::Grease => "What's worse than slipping? Slipping while on fire.",
            Spell::FogCloud => "Can't hit what you can't see.",
            Spell::BattleHymn => "Nothing motivates like a good war song.",
            Spell::HealingPlume => "Stand in the glowy green zone. Trust the wizard.",
            Spell::BerserkerRage => "Hit harder. Get hit harder. Worth it.",
            Spell::Banishment => "Poof. Gone. Back in a minute.",
            Spell::Polymorph => "Baa.",
            Spell::ArcaneCrystal => {
                "A prism for the ambitious. What goes in comes out... everywhere."
            }
            Spell::Dispel => "Sometimes the best magic is no magic at all.",
            Spell::MindControl => "Why fight your enemies when they can fight each other?",
        }
    }

    /// Returns the asset path for this spell's icon image, if one exists.
    pub const fn icon_path(&self) -> Option<&'static str> {
        match self {
            Spell::Fireball => Some("images/icons/fireball_icon.png"),
            Spell::MagicMissile => Some("images/icons/magic_missile_icon.png"),
            Spell::Disintegrate => Some("images/icons/disintegrate_icon.png"),
            Spell::WallOfFire => Some("images/icons/wall_of_fire_icon.png"),
            Spell::MeteorFall => Some("images/icons/meteor_fall_icon.png"),
            Spell::GuardianCircle => Some("images/icons/guardian_circle_icon.png"),
            Spell::ChainLightning => Some("images/icons/chain_lightning_icon.png"),
            Spell::FingerOfDeath => Some("images/icons/finger_of_death_icon.png"),
            Spell::RaiseTheDead => Some("images/icons/raise_the_dead_icon.png"),
            Spell::Teleport => Some("images/icons/teleport_icon.png"),
            Spell::Squall => Some("images/icons/squall_icon.png"),
            Spell::WallOfStone => Some("images/icons/wall_of_stone_icon.png"),
            Spell::Entangle => Some("images/icons/entangle_icon.png"),
            Spell::Haste => Some("images/icons/haste_icon.png"),
            Spell::SpikeGrowth => Some("images/icons/spike_growth_icon.png"),
            Spell::LightningRod => Some("images/icons/lightning_rod_icon.png"),
            Spell::BlackHole => Some("images/icons/black_hole_icon.png"),
            Spell::Telekinesis => Some("images/icons/telekinesis_icon.png"),
            Spell::MarkOfDeath => Some("images/icons/mark_of_death_icon.png"),
            Spell::Sleep => Some("images/icons/sleep_icon.png"),
            Spell::PlagueWind => Some("images/icons/plague_wind_icon.png"),
            Spell::BattleHymn => Some("images/icons/battle_hymn_icon.png"),
            Spell::FogCloud => Some("images/icons/fog_cloud_icon.png"),
            Spell::Grease => Some("images/icons/grease_icon.png"),
            Spell::HealingPlume => Some("images/icons/healing_plume_icon.png"),
            Spell::BerserkerRage => Some("images/icons/berserker_rage_icon.png"),
            Spell::Banishment => Some("images/icons/banishment_icon.png"),
            Spell::Polymorph => Some("images/icons/polymorph_icon.png"),
            Spell::ArcaneCrystal => Some("images/icons/arcane_crystal_icon.png"),
            Spell::Dispel => Some("images/icons/dispel_icon.png"),
            Spell::MindControl => Some("images/icons/mind_control_icon.png"),
        }
    }

    /// Returns the control instructions for this spell.
    pub const fn instructions(&self) -> &'static str {
        match self {
            Spell::MagicMissile => "Click to fire",
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
            Spell::Sleep => "Click and hold to place",
            Spell::Grease => "Click and hold to place",
            Spell::FogCloud => "Click and hold to place",
            Spell::BattleHymn => "Click and hold to place",
            Spell::HealingPlume => "Click and hold to place",
            Spell::BerserkerRage => "Click and hold to place",
            Spell::Banishment => "Click and hold to cast",
            Spell::Polymorph => "Click and hold to cast",
            Spell::ArcaneCrystal => "Click and hold to place",
            Spell::Dispel => "Click to fire",
            Spell::MindControl => "Click and hold to cast",
        }
    }

    /// Returns the PrimedSpell configuration for this spell.
    pub const fn primed_config(self) -> PrimedSpell {
        use crate::game::units::wizard::spells::{
            arcane_crystal_constants, banishment_constants, battle_hymn_constants,
            berserker_rage_constants, black_hole_constants, chain_lightning_constants,
            disintegrate_constants, dispel_constants, entangle_constants,
            finger_of_death_constants, fireball_constants, fog_cloud_constants, grease_constants,
            guardian_circle_constants, haste_constants, healing_plume_constants,
            lightning_rod_constants, magic_missile_constants, mark_of_death_constants,
            meteor_fall_constants, mind_control_constants, plague_wind_constants,
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
            Spell::Sleep => sleep_constants::PRIMED_SLEEP,
            Spell::Grease => grease_constants::PRIMED_GREASE,
            Spell::FogCloud => fog_cloud_constants::PRIMED_FOG_CLOUD,
            Spell::BattleHymn => battle_hymn_constants::PRIMED_BATTLE_HYMN,
            Spell::HealingPlume => healing_plume_constants::PRIMED_HEALING_PLUME,
            Spell::BerserkerRage => berserker_rage_constants::PRIMED_BERSERKER_RAGE,
            Spell::Banishment => banishment_constants::PRIMED_BANISHMENT,
            Spell::Polymorph => polymorph_constants::PRIMED_POLYMORPH,
            Spell::ArcaneCrystal => arcane_crystal_constants::PRIMED_ARCANE_CRYSTAL,
            Spell::Dispel => dispel_constants::PRIMED_DISPEL,
            Spell::MindControl => mind_control_constants::PRIMED_MIND_CONTROL,
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
            Spell::SpikeGrowth => DamageType::Poison,
            Spell::LightningRod => DamageType::Electric,
            Spell::Telekinesis => DamageType::Force,
            Spell::MeteorFall => DamageType::Fire,
            Spell::MarkOfDeath => DamageType::Necrotic,
            Spell::PlagueWind => DamageType::Poison,
            Spell::Sleep => DamageType::Force,
            Spell::Grease => DamageType::Nature,
            Spell::FogCloud => DamageType::Frost,
            Spell::BattleHymn => DamageType::Force,
            Spell::HealingPlume => DamageType::Nature,
            Spell::BerserkerRage => DamageType::Force,
            Spell::Banishment => DamageType::Force,
            Spell::Polymorph => DamageType::Nature,
            Spell::ArcaneCrystal => DamageType::Force,
            Spell::Dispel => DamageType::Force,
            Spell::MindControl => DamageType::Force,
        }
    }

    /// Returns the Arcane Insight cost to research this spell.
    /// Default spells have cost 0 — they start unlocked. See `default_unlocked()`.
    pub const fn research_cost(&self) -> u32 {
        match self {
            Spell::MagicMissile
            | Spell::Telekinesis
            | Spell::GuardianCircle
            | Spell::Entangle => 0,
            // Root spells (immediately researchable)
            Spell::Disintegrate => 30,
            Spell::Grease => 30,
            Spell::ChainLightning => 30,
            Spell::FingerOfDeath => 30,
            Spell::WallOfStone => 30,
            // Second-tier (requires predecessor)
            Spell::Fireball => 50,
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
            Spell::PlagueWind => 80,
            // Fourth-tier (capstones)
            Spell::MeteorFall => 100,
            // Misc (require N total spells researched)
            Spell::Squall => 70,
            Spell::FogCloud => 60,
            Spell::Sleep => 80,
            Spell::Banishment => 90,
            Spell::BlackHole => 120,
            Spell::Polymorph => 100,
            Spell::ArcaneCrystal => 80,
            Spell::Dispel => 40,
            Spell::MindControl => 80,
        }
    }

    /// Returns the prerequisite spell that must be researched first, if any.
    pub const fn prerequisite(&self) -> Option<Spell> {
        match self {
            // Offense tree: MagicMissile (root)
            Spell::Disintegrate => Some(Spell::MagicMissile),
            Spell::ChainLightning => Some(Spell::MagicMissile),
            Spell::FingerOfDeath => Some(Spell::MagicMissile),
            Spell::PlagueWind => Some(Spell::MagicMissile),
            Spell::Fireball => Some(Spell::Disintegrate),
            Spell::MeteorFall => Some(Spell::Fireball),
            Spell::LightningRod => Some(Spell::ChainLightning),
            Spell::MarkOfDeath => Some(Spell::FingerOfDeath),
            // Control tree: Entangle (root)
            Spell::Grease => Some(Spell::Entangle),
            Spell::SpikeGrowth => Some(Spell::Entangle),
            Spell::Sleep => Some(Spell::Entangle),
            Spell::WallOfFire => Some(Spell::Grease),
            Spell::WallOfStone => Some(Spell::SpikeGrowth),
            Spell::Squall => Some(Spell::SpikeGrowth),
            Spell::MindControl => Some(Spell::Sleep),
            Spell::Polymorph => Some(Spell::Sleep),
            Spell::BlackHole => Some(Spell::Polymorph),
            // Support tree: GuardianCircle (root)
            Spell::BattleHymn => Some(Spell::GuardianCircle),
            Spell::FogCloud => Some(Spell::GuardianCircle),
            Spell::BerserkerRage => Some(Spell::GuardianCircle),
            Spell::HealingPlume => Some(Spell::BattleHymn),
            Spell::Haste => Some(Spell::BattleHymn),
            Spell::Teleport => Some(Spell::FogCloud),
            Spell::RaiseTheDead => Some(Spell::BerserkerRage),
            // Utility tree: Telekinesis (root)
            Spell::Dispel => Some(Spell::Telekinesis),
            Spell::Banishment => Some(Spell::Telekinesis),
            Spell::ArcaneCrystal => Some(Spell::Telekinesis),
            _ => None,
        }
    }

    /// Returns the minimum number of total spells that must be researched
    /// before this spell becomes available (0 = no requirement).
    /// Used for miscellaneous spells not in a prerequisite chain.
    pub const fn required_total_spells(&self) -> u32 {
        // All spells are now in prerequisite chains; no gate requirements needed.
        0
    }

    /// Returns true if this spell is allowed for the Shepherd wizard type.
    pub const fn is_shepherd_allowed(&self) -> bool {
        matches!(
            self,
            Spell::WallOfStone
                | Spell::Entangle
                | Spell::Sleep
                | Spell::Polymorph
                | Spell::MindControl
                | Spell::GuardianCircle
                | Spell::Haste
                | Spell::Teleport
                | Spell::RaiseTheDead
                | Spell::BattleHymn
                | Spell::HealingPlume
                | Spell::FogCloud
                | Spell::BerserkerRage
                | Spell::Banishment
                | Spell::Dispel
                | Spell::Telekinesis
        )
    }

    /// Returns true if this spell is researchable (not a default spell).
    pub const fn is_researchable(&self) -> bool {
        self.research_cost() > 0
    }

    /// Returns all researchable spells (those with a non-zero research cost).
    pub fn researchable() -> Vec<Spell> {
        Spell::all()
            .iter()
            .copied()
            .filter(|s| s.is_researchable())
            .collect()
    }

    /// The root spells that are unlocked by default for new players,
    /// in canonical tree order: Offense, Control, Support, Utility.
    /// Used as the starting action bar layout for new wizards.
    pub const fn default_unlocked() -> [Spell; 4] {
        [
            Spell::MagicMissile,
            Spell::Entangle,
            Spell::GuardianCircle,
            Spell::Telekinesis,
        ]
    }

    /// In-scope status effects this spell applies during its base or talented
    /// behavior. Surfaced under "Status effects" in the spell book and
    /// compendium. Effects that only fire under a specific talent path are
    /// still listed — players can see the full menu of possibilities.
    pub(crate) const fn status_effects(&self) -> &'static [StatusEffectKind] {
        match self {
            // Burning
            Spell::Fireball
            | Spell::WallOfFire
            | Spell::Disintegrate
            | Spell::Grease
            | Spell::MeteorFall => &[StatusEffectKind::Burning],

            // Frost / Frozen — Squall accumulates frost; Permafrost talent freezes.
            Spell::Squall => &[StatusEffectKind::Frost, StatusEffectKind::Frozen],

            // Slow-only
            Spell::BlackHole => &[StatusEffectKind::Slowed],

            // Lightning — apply a Shocked charge that primes chain arcs.
            Spell::ChainLightning => &[StatusEffectKind::Shocked],
            Spell::LightningRod => &[StatusEffectKind::Shocked],

            // Poison (+ slow)
            Spell::PlagueWind => &[StatusEffectKind::Poisoned, StatusEffectKind::Slowed],
            Spell::SpikeGrowth => &[StatusEffectKind::Poisoned],

            // Hard CC
            Spell::Entangle => &[StatusEffectKind::Rooted, StatusEffectKind::Slowed],
            Spell::Sleep => &[StatusEffectKind::Slept],
            Spell::Polymorph => &[StatusEffectKind::Polymorphed],
            Spell::Banishment => &[StatusEffectKind::Banished],
            Spell::MarkOfDeath => &[StatusEffectKind::Marked],

            // No in-scope status effect (utility, pure damage, support).
            Spell::MagicMissile
            | Spell::WallOfStone
            | Spell::GuardianCircle
            | Spell::FingerOfDeath
            | Spell::RaiseTheDead
            | Spell::Teleport
            | Spell::Haste
            | Spell::Telekinesis
            | Spell::FogCloud
            | Spell::BattleHymn
            | Spell::HealingPlume
            | Spell::BerserkerRage
            | Spell::ArcaneCrystal
            | Spell::Dispel
            | Spell::MindControl => &[],
        }
    }
}
