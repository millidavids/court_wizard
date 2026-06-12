use bevy::prelude::*;

use super::constants::*;
use super::effect::BrewEffect;

/// Category for grouping ingredients in the cauldron UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngredientCategory {
    Offense,
    Control,
    Support,
    Utility,
}

impl IngredientCategory {
    /// Returns the display name for this category.
    pub const fn display_name(&self) -> &'static str {
        match self {
            IngredientCategory::Offense => "Offense",
            IngredientCategory::Control => "Control",
            IngredientCategory::Support => "Support",
            IngredientCategory::Utility => "Utility",
        }
    }

    /// Returns all categories in display order.
    pub const fn all() -> &'static [IngredientCategory] {
        &[
            IngredientCategory::Offense,
            IngredientCategory::Control,
            IngredientCategory::Support,
            IngredientCategory::Utility,
        ]
    }
}

/// An ingredient that can be added to a brew.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ingredient {
    Lavender,
    Mugwort,
    Yarrow,
    Mistletoe,
    Vervain,
    Wormwood,
    BlueLotus,
    Meadowsweet,
    Valerian,
    NatronSalt,
    LapisLazuli,
    Henbane,
    Frankincense,
    Amber,
    RavenFeather,
    MandrakeRoot,
    RowanBerry,
    DragonsBlood,
    PhilosophersStone,
}

/// Static configuration for an ingredient.
pub struct IngredientConfig {
    /// Display name shown in the UI.
    pub name: &'static str,
    /// Flavor text describing the ingredient's appearance and lore.
    pub flavor_text: &'static str,
    /// Short functional description of the gameplay effect.
    pub functional_description: &'static str,
    /// The effect this ingredient contributes at full strength.
    pub effect: BrewEffect,
    /// Visual color based on the real-life ingredient.
    pub color: Color,
}

impl Ingredient {
    /// Returns the category this ingredient belongs to.
    pub const fn category(&self) -> IngredientCategory {
        match self {
            // Offense: damage/attack focused
            Ingredient::Mugwort
            | Ingredient::Mistletoe
            | Ingredient::Henbane
            | Ingredient::DragonsBlood
            | Ingredient::Frankincense => IngredientCategory::Offense,
            // Control: slows/CC/area
            Ingredient::Valerian
            | Ingredient::Meadowsweet
            | Ingredient::BlueLotus
            | Ingredient::RavenFeather => IngredientCategory::Control,
            // Support: healing/defense
            Ingredient::Yarrow
            | Ingredient::Wormwood
            | Ingredient::NatronSalt
            | Ingredient::RowanBerry => IngredientCategory::Support,
            // Utility: mana/duration/meta
            Ingredient::Lavender
            | Ingredient::Vervain
            | Ingredient::LapisLazuli
            | Ingredient::Amber
            | Ingredient::MandrakeRoot
            | Ingredient::PhilosophersStone => IngredientCategory::Utility,
        }
    }

    /// Returns the static configuration for this ingredient.
    pub fn config(&self) -> &'static IngredientConfig {
        match self {
            Ingredient::Lavender => &LAVENDER_CONFIG,
            Ingredient::Mugwort => &MUGWORT_CONFIG,
            Ingredient::Yarrow => &YARROW_CONFIG,
            Ingredient::Mistletoe => &MISTLETOE_CONFIG,
            Ingredient::Vervain => &VERVAIN_CONFIG,
            Ingredient::Wormwood => &WORMWOOD_CONFIG,
            Ingredient::BlueLotus => &BLUE_LOTUS_CONFIG,
            Ingredient::Meadowsweet => &MEADOWSWEET_CONFIG,
            Ingredient::Valerian => &VALERIAN_CONFIG,
            Ingredient::NatronSalt => &NATRON_SALT_CONFIG,
            Ingredient::LapisLazuli => &LAPIS_LAZULI_CONFIG,
            Ingredient::Henbane => &HENBANE_CONFIG,
            Ingredient::Frankincense => &FRANKINCENSE_CONFIG,
            Ingredient::Amber => &AMBER_CONFIG,
            Ingredient::RavenFeather => &RAVEN_FEATHER_CONFIG,
            Ingredient::MandrakeRoot => &MANDRAKE_ROOT_CONFIG,
            Ingredient::RowanBerry => &ROWAN_BERRY_CONFIG,
            Ingredient::DragonsBlood => &DRAGONS_BLOOD_CONFIG,
            Ingredient::PhilosophersStone => &PHILOSOPHERS_STONE_CONFIG,
        }
    }

    /// Returns all available ingredients.
    /// Stable on-disk key for save data. Pinned to the variant name explicitly so
    /// that renaming a variant (or changing the `Debug` derive) can never silently
    /// orphan existing player saves. A unit test asserts this stays equal to the
    /// `Debug` representation, which is the format already on disk.
    pub fn save_key(&self) -> &'static str {
        match self {
            Self::Lavender => "Lavender",
            Self::Mugwort => "Mugwort",
            Self::Yarrow => "Yarrow",
            Self::Mistletoe => "Mistletoe",
            Self::Vervain => "Vervain",
            Self::Wormwood => "Wormwood",
            Self::BlueLotus => "BlueLotus",
            Self::Meadowsweet => "Meadowsweet",
            Self::Valerian => "Valerian",
            Self::NatronSalt => "NatronSalt",
            Self::LapisLazuli => "LapisLazuli",
            Self::Henbane => "Henbane",
            Self::Frankincense => "Frankincense",
            Self::Amber => "Amber",
            Self::RavenFeather => "RavenFeather",
            Self::MandrakeRoot => "MandrakeRoot",
            Self::RowanBerry => "RowanBerry",
            Self::DragonsBlood => "DragonsBlood",
            Self::PhilosophersStone => "PhilosophersStone",
        }
    }

    pub const fn all() -> &'static [Ingredient] {
        &[
            Ingredient::Lavender,
            Ingredient::Mugwort,
            Ingredient::Yarrow,
            Ingredient::Mistletoe,
            Ingredient::Vervain,
            Ingredient::Wormwood,
            Ingredient::BlueLotus,
            Ingredient::Meadowsweet,
            Ingredient::Valerian,
            Ingredient::NatronSalt,
            Ingredient::LapisLazuli,
            Ingredient::Henbane,
            Ingredient::Frankincense,
            Ingredient::Amber,
            Ingredient::RavenFeather,
            Ingredient::MandrakeRoot,
            Ingredient::RowanBerry,
            Ingredient::DragonsBlood,
        ]
    }

    /// Returns the display name for this ingredient.
    pub fn name(&self) -> &'static str {
        self.config().name
    }

    /// Returns the full description (flavor text + functional) for this ingredient.
    pub fn description(&self) -> String {
        let config = self.config();
        format!(
            "{}\n\n{}",
            config.flavor_text, config.functional_description
        )
    }

    /// Returns only the functional gameplay description.
    pub fn functional_description(&self) -> &'static str {
        self.config().functional_description
    }

    /// Returns flavor text shown on the progress screen when the ingredient is locked.
    pub const fn locked_description(&self) -> &'static str {
        match self {
            Ingredient::Lavender => "Smells nice. Does something magical. Probably.",
            Ingredient::Mugwort => "Not actually related to mugs. Or warts.",
            Ingredient::Yarrow => "Ancient herbalists swore by it. They were mostly right.",
            Ingredient::Mistletoe => "Kiss under it or kill with it. Hedge wizards did both.",
            Ingredient::Vervain => "Enchanters weave fate. This herb unravels it.",
            Ingredient::Wormwood => "Bitter to taste, bitter for your enemies.",
            Ingredient::BlueLotus => "Ancient alchemists saw gods. You'll see bigger explosions.",
            Ingredient::Meadowsweet => "Warriors ran faster with this. Don't ask how.",
            Ingredient::Valerian => "Makes enemies drowsy. Makes you victorious.",
            Ingredient::NatronSalt => {
                "Preserved archmages for millennia. Should work for soldiers."
            }
            Ingredient::LapisLazuli => {
                "Mages carved it into wands. The mana practically leaks out."
            }
            Ingredient::Henbane => "Berserkers ate it before battle. Side effects include victory.",
            Ingredient::Frankincense => {
                "Temple smoke that makes spells hit harder. Gods not included."
            }
            Ingredient::Amber => "Traps time itself. Your buffs will thank you.",
            Ingredient::RavenFeather => {
                "Plucked from Odin's messenger. Everything hits differently."
            }
            Ingredient::MandrakeRoot => "Screams when pulled. Screams louder in a cauldron.",
            Ingredient::RowanBerry => "Witches feared this tree. Now it works for you.",
            Ingredient::DragonsBlood => {
                "Not actual dragon blood. Probably. The merchant was unclear."
            }
            Ingredient::PhilosophersStone => {
                "The ultimate prize. Legends say it perfects any brew."
            }
        }
    }

    /// Returns true if this ingredient is the Philosopher's Stone.
    pub const fn is_philosophers_stone(&self) -> bool {
        matches!(self, Ingredient::PhilosophersStone)
    }
}
