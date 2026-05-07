//! Unit-type enum and compendium sprite spec.

use bevy::prelude::*;

/// All trackable unit types in the game.
///
/// Variants are ordered by typical encounter / unlock order:
/// defenders (default unlocked), then attackers in start-tier order,
/// then bosses in the level they first appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitType {
    // Defenders (default unlocked)
    Infantry,
    Archer,
    King,
    KingsGuard,
    // Attackers (unlocked on encounter, ordered by first-appearance tier)
    Elite,     // tier 1
    Assassin,  // tier 1
    Brute,     // tier 2
    Commander, // tier 2
    Dispeller, // tier 2
    Aerialist, // tier 2
    Healer,    // tier 3
    Shielder,  // tier 3
    // Bosses (ordered by level: 5, 10, 15, 20, 25)
    Ogre,
    Lich,
    DarkMage,
    Hag,
    Ray,
}

impl UnitType {
    /// Returns all unit type variants in encounter / unlock order.
    pub fn all() -> &'static [UnitType] {
        &[
            UnitType::Infantry,
            UnitType::Archer,
            UnitType::King,
            UnitType::KingsGuard,
            UnitType::Elite,
            UnitType::Assassin,
            UnitType::Brute,
            UnitType::Commander,
            UnitType::Dispeller,
            UnitType::Aerialist,
            UnitType::Healer,
            UnitType::Shielder,
            UnitType::Ogre,
            UnitType::Lich,
            UnitType::DarkMage,
            UnitType::Hag,
            UnitType::Ray,
        ]
    }

    /// Display name for the UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            UnitType::Infantry => "Infantry",
            UnitType::Archer => "Archer",
            UnitType::King => "King",
            UnitType::KingsGuard => "King's Guard",
            UnitType::Elite => "Elite",
            UnitType::Assassin => "Assassin",
            UnitType::Brute => "Brute",
            UnitType::Commander => "Commander",
            UnitType::Dispeller => "Dispeller",
            UnitType::Aerialist => "Aerialist",
            UnitType::Healer => "Healer",
            UnitType::Shielder => "Shielder",
            UnitType::Ogre => "Ogre",
            UnitType::Lich => "Lich",
            UnitType::DarkMage => "Dark Mage",
            UnitType::Hag => "Hag",
            UnitType::Ray => "Ray",
        }
    }

    /// Short description of the unit.
    pub const fn description(&self) -> &'static str {
        match self {
            UnitType::Infantry => "Melee fighters forming the front line of defense.",
            UnitType::Archer => "Ranged soldiers picking off targets from afar.",
            UnitType::King => "The leader you must protect at all costs.",
            UnitType::KingsGuard => "Elite warriors sworn to defend the King.",
            UnitType::Elite => "Enhanced soldiers with bonus health, damage, and speed.",
            UnitType::Assassin => "Fast flankers that slip past infantry to strike archers.",
            UnitType::Brute => "Heavy-hitting melee fighters that hit like a truck.",
            UnitType::Commander => "Officers that buff nearby allies with damage and speed auras.",
            UnitType::Dispeller => "Anti-magic units that remove your spell effects.",
            UnitType::Aerialist => "Flying attackers that swoop over walls and strike from above.",
            UnitType::Healer => "Support units that restore health to wounded allies.",
            UnitType::Shielder => "Support units that shield allies from your spells.",
            UnitType::Ogre => "A massive beast that grows stronger as the fight goes on.",
            UnitType::Lich => "An undead sorcerer who grows stronger from death itself.",
            UnitType::DarkMage => {
                "A telegraphed-AoE spellcaster who rains meteors and teleports across the field."
            }
            UnitType::Hag => "Ancient witches with devastating magical abilities.",
            UnitType::Ray => {
                "A heart-bound horror whose every beat warps the battlefield with dark pulses."
            }
        }
    }

    /// Flavor text shown in the compendium.
    pub const fn flavor_text(&self) -> &'static str {
        match self {
            UnitType::Infantry => "They signed up for this. Probably.",
            UnitType::Archer => "They never miss. Except when they do.",
            UnitType::King => {
                "Heavy is the head that wears the crown. Heavier when fireballs are involved."
            }
            UnitType::KingsGuard => {
                "Sworn to protect, trained to intimidate, paid to stand very still."
            }
            UnitType::Elite => {
                "Better than regular soldiers in every measurable way. They won't let you forget it."
            }
            UnitType::Assassin => "They don't fight fair. That's the whole point.",
            UnitType::Brute => "What they lack in strategy, they make up for in sheer mass.",
            UnitType::Commander => {
                "Barking orders from behind the front line, as tradition demands."
            }
            UnitType::Dispeller => "Your spells mean nothing to them. Take it personally.",
            UnitType::Aerialist => "Death from above. Way, way above.",
            UnitType::Healer => "The only unit the enemy army actually values. Unfortunately.",
            UnitType::Shielder => {
                "Handing out magical umbrellas like party favors. How thoughtful."
            }
            UnitType::Ogre => "Started the fight angry. It only gets worse from there.",
            UnitType::Lich => "Every fallen soldier is just another name on his roster.",
            UnitType::DarkMage => "Robes, hood, staff, attitude. The whole package.",
            UnitType::Hag => "Three sisters who share one terrible disposition.",
            UnitType::Ray => "All heart, no mercy. Mostly heart, actually.",
        }
    }

    /// Whether this unit is unlocked by default (defenders).
    pub const fn is_default_unlocked(&self) -> bool {
        matches!(
            self,
            UnitType::Infantry | UnitType::Archer | UnitType::King | UnitType::KingsGuard
        )
    }

    /// Team label for display.
    pub const fn team_label(&self) -> &'static str {
        match self {
            UnitType::Infantry | UnitType::Archer | UnitType::King | UnitType::KingsGuard => {
                "Defender"
            }
            UnitType::Elite
            | UnitType::Assassin
            | UnitType::Brute
            | UnitType::Commander
            | UnitType::Dispeller
            | UnitType::Aerialist
            | UnitType::Healer
            | UnitType::Shielder => "Attacker",
            UnitType::Ogre
            | UnitType::Lich
            | UnitType::DarkMage
            | UnitType::Hag
            | UnitType::Ray => "Boss",
        }
    }

    /// Locked description hint shown when the unit hasn't been encountered.
    pub const fn locked_description(&self) -> &'static str {
        match self {
            UnitType::Infantry => "A common defender.",
            UnitType::Archer => "A ranged defender.",
            UnitType::King => "The one you protect.",
            UnitType::KingsGuard => "Royal bodyguards.",
            UnitType::Elite => "The enemy is adapting.",
            UnitType::Assassin => "Shadows move faster than they should.",
            UnitType::Brute => "Something big is coming...",
            UnitType::Commander => "Someone is giving orders out there.",
            UnitType::Dispeller => "Your magic feels weaker somehow.",
            UnitType::Aerialist => "Something circles overhead.",
            UnitType::Healer => "The wounded keep getting back up.",
            UnitType::Shielder => "Something is protecting the enemy.",
            UnitType::Ogre => "The ground trembles.",
            UnitType::Lich => "The dead whisper of a master.",
            UnitType::DarkMage => "A robed figure watches from the back lines.",
            UnitType::Hag => "Dark magic stirs in the distance.",
            UnitType::Ray => "A faint, steady heartbeat echoes from somewhere unseen.",
        }
    }

    /// Portrait spec for the compendium detail panel.
    pub(crate) fn compendium_sprite(&self) -> CompendiumSpriteSpec {
        use CompendiumSpriteSpec::*;
        match self {
            UnitType::Infantry => atlas_infantry(Color::srgb(1.3, 1.3, 1.5), 1.0),
            UnitType::KingsGuard => atlas_infantry(Color::srgb(1.2, 0.45, 0.35), 1.2),
            UnitType::King => atlas_infantry(Color::srgb(1.4, 1.15, 0.4), 1.5),
            UnitType::Brute => atlas_infantry(Color::srgb(0.7, 0.2, 1.0), 2.5),
            UnitType::Elite => atlas_infantry(Color::srgb(1.0, 0.15, 0.1), 1.3),
            UnitType::Commander => atlas_infantry(Color::srgb(1.0, 0.6, 0.1), 1.6),

            UnitType::Archer => atlas_64(
                "images/sprite_sheets/archer-walking_9-frames.png",
                Color::srgb(0.75, 0.65, 0.65),
                1.0,
            ),
            UnitType::Healer => atlas_64(
                "images/sprite_sheets/healer-walking_9-frames.png",
                Color::WHITE,
                1.0,
            ),
            UnitType::Dispeller => atlas_64(
                "images/sprite_sheets/dispeller-walking_9-frames.png",
                Color::WHITE,
                1.0,
            ),
            UnitType::Shielder => atlas_64(
                "images/sprite_sheets/shielder-walking_9-frames.png",
                Color::WHITE,
                1.0,
            ),
            UnitType::Assassin => atlas_64(
                "images/sprite_sheets/assassin-walking_9-frames.png",
                Color::WHITE,
                1.0,
            ),
            UnitType::Aerialist => atlas_64(
                "images/sprite_sheets/aerialist-flying_2-frames.png",
                Color::WHITE,
                1.0,
            ),

            UnitType::Ogre => Static {
                path: "images/static_sprites/ogre.png",
                size_multiplier: 2.0,
            },
            UnitType::Lich => Static {
                path: "images/static_sprites/lich.png",
                size_multiplier: 2.0,
            },
            UnitType::DarkMage => Static {
                path: "images/static_sprites/dark_mage.png",
                size_multiplier: 2.0,
            },
            UnitType::Hag => Static {
                path: "images/static_sprites/hag.png",
                size_multiplier: 2.0,
            },
            UnitType::Ray => Static {
                path: "images/static_sprites/ray.png",
                size_multiplier: 2.0,
            },
        }
    }
}

/// Specifies how to display a unit's portrait in the compendium detail panel.
#[derive(Clone, Copy)]
pub(crate) enum CompendiumSpriteSpec {
    /// Use a pre-made static portrait image.
    Static {
        path: &'static str,
        size_multiplier: f32,
    },
    /// Crop frame 0 from a walking sprite sheet.
    Atlas {
        path: &'static str,
        sheet_size: (u32, u32),
        frame_px: u32,
        tint: Color,
        /// Display size relative to baseline infantry (1.0 = DETAIL_UNIT_ICON_SIZE).
        size_multiplier: f32,
    },
}

fn atlas_64(path: &'static str, tint: Color, size_multiplier: f32) -> CompendiumSpriteSpec {
    CompendiumSpriteSpec::Atlas {
        path,
        sheet_size: (832, 256),
        frame_px: 64,
        tint,
        size_multiplier,
    }
}

fn atlas_infantry(tint: Color, size_multiplier: f32) -> CompendiumSpriteSpec {
    atlas_64(
        "images/sprite_sheets/infantry-walking_9-frames.png",
        tint,
        size_multiplier,
    )
}
