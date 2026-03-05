pub(crate) mod constants;

use bevy::prelude::*;

use constants::*;

/// A single effect that an ingredient/brew applies when active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrewEffect {
    /// Multiplies wizard mana regeneration rate.
    ManaRegenMultiplier(f32),
    /// Multiplies spell power/empowerment.
    SpellPowerMultiplier(f32),
    /// Heals all defender units by this amount per second.
    DefenderHealPerSecond(f32),
    /// Multiplies wizard cast speed (higher = faster).
    CastSpeedMultiplier(f32),
    /// Multiplies spell area of effect.
    SpellRangeMultiplier(f32),
    /// Bonus damage percentage for defender units (0.5 = +50%).
    DefenderDamageBonus(f32),
    /// Damage resistance percentage for defender units (0.25 = 25% reduction).
    DamageResistancePercent(f32),
    /// Speed bonus percentage for defender units (0.3 = +30%).
    DefenderSpeedBonus(f32),
    /// Speed reduction percentage for attacker/undead units (0.25 = 25% slow).
    AttackerSlowPercent(f32),
    /// Grants temporary hit points to defenders per second.
    DefenderShieldPerSecond(f32),
    /// Multiplies wizard maximum mana capacity.
    MaxManaMultiplier(f32),
    /// Multiplies defender attack speed (higher = faster).
    AttackSpeedMultiplier(f32),
    /// Multiplies buff duration from brews.
    BuffDurationMultiplier(f32),
    /// Flat bonus to defender effectiveness.
    EffectivenessBonus(f32),
}

impl BrewEffect {
    /// Returns a human-readable description of this effect, e.g. "Mana Regen +100%".
    pub fn display_text(&self) -> String {
        match self {
            BrewEffect::ManaRegenMultiplier(v) => format!("Mana regen: +{:.0}%", (v - 1.0) * 100.0),
            BrewEffect::SpellPowerMultiplier(v) => {
                format!("Spell power: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::DefenderHealPerSecond(v) => format!("Defender healing: {:.1} HP/s", v),
            BrewEffect::CastSpeedMultiplier(v) => {
                format!("Cast speed: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::SpellRangeMultiplier(v) => {
                format!("Spell area: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::DefenderDamageBonus(v) => format!("Defender damage: +{:.0}%", v * 100.0),
            BrewEffect::DamageResistancePercent(v) => {
                format!("Damage resistance: {:.0}%", v * 100.0)
            }
            BrewEffect::DefenderSpeedBonus(v) => format!("Defender speed: +{:.0}%", v * 100.0),
            BrewEffect::AttackerSlowPercent(v) => format!("Enemy slow: {:.0}%", v * 100.0),
            BrewEffect::DefenderShieldPerSecond(v) => format!("Defender shield: {:.1} HP/s", v),
            BrewEffect::MaxManaMultiplier(v) => format!("Max mana: +{:.0}%", (v - 1.0) * 100.0),
            BrewEffect::AttackSpeedMultiplier(v) => {
                format!("Attack speed: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::BuffDurationMultiplier(v) => {
                format!("Buff duration: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::EffectivenessBonus(v) => {
                format!("Defender effectiveness: +{:.0}%", v * 100.0)
            }
        }
    }

    /// Returns a short 1-2 character abbreviation for use in buff tracker boxes.
    pub const fn abbreviation(&self) -> &'static str {
        match self {
            BrewEffect::ManaRegenMultiplier(_) => "MR",
            BrewEffect::SpellPowerMultiplier(_) => "SP",
            BrewEffect::DefenderHealPerSecond(_) => "HL",
            BrewEffect::CastSpeedMultiplier(_) => "CS",
            BrewEffect::SpellRangeMultiplier(_) => "AE",
            BrewEffect::DefenderDamageBonus(_) => "DD",
            BrewEffect::DamageResistancePercent(_) => "DR",
            BrewEffect::DefenderSpeedBonus(_) => "DS",
            BrewEffect::AttackerSlowPercent(_) => "SL",
            BrewEffect::DefenderShieldPerSecond(_) => "SH",
            BrewEffect::MaxManaMultiplier(_) => "MM",
            BrewEffect::AttackSpeedMultiplier(_) => "AS",
            BrewEffect::BuffDurationMultiplier(_) => "BD",
            BrewEffect::EffectivenessBonus(_) => "EF",
        }
    }

    /// Scales an effect's magnitude by a factor.
    /// Multiplier effects (base 1.0): 1.0 + (value - 1.0) * factor
    /// Flat effects (base 0.0): value * factor
    pub fn scale(self, factor: f32) -> BrewEffect {
        match self {
            BrewEffect::ManaRegenMultiplier(v) => {
                BrewEffect::ManaRegenMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::SpellPowerMultiplier(v) => {
                BrewEffect::SpellPowerMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::CastSpeedMultiplier(v) => {
                BrewEffect::CastSpeedMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::SpellRangeMultiplier(v) => {
                BrewEffect::SpellRangeMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::MaxManaMultiplier(v) => {
                BrewEffect::MaxManaMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::AttackSpeedMultiplier(v) => {
                BrewEffect::AttackSpeedMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::BuffDurationMultiplier(v) => {
                BrewEffect::BuffDurationMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::DefenderHealPerSecond(v) => BrewEffect::DefenderHealPerSecond(v * factor),
            BrewEffect::DefenderDamageBonus(v) => BrewEffect::DefenderDamageBonus(v * factor),
            BrewEffect::DamageResistancePercent(v) => {
                BrewEffect::DamageResistancePercent(v * factor)
            }
            BrewEffect::DefenderSpeedBonus(v) => BrewEffect::DefenderSpeedBonus(v * factor),
            BrewEffect::AttackerSlowPercent(v) => BrewEffect::AttackerSlowPercent(v * factor),
            BrewEffect::DefenderShieldPerSecond(v) => {
                BrewEffect::DefenderShieldPerSecond(v * factor)
            }
            BrewEffect::EffectivenessBonus(v) => BrewEffect::EffectivenessBonus(v * factor),
        }
    }
}

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
}

/// Static configuration for an ingredient.
pub struct IngredientConfig {
    /// Display name shown in the UI.
    pub name: &'static str,
    /// Short description of the ingredient's effect.
    pub description: &'static str,
    /// The effect this ingredient contributes at full strength.
    pub effect: BrewEffect,
    /// Visual color based on the real-life ingredient.
    pub color: Color,
}

/// A hidden combo bonus triggered when specific ingredients are combined.
pub struct ComboBonus {
    /// Display name of the combo.
    pub name: &'static str,
    /// Required ingredients (all must be present in the recipe).
    pub required: &'static [Ingredient],
    /// Bonus effects granted (undiluted).
    pub bonus_effects: &'static [BrewEffect],
    /// Flavor text description.
    pub description: &'static str,
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
            | Ingredient::MandrakeRoot => IngredientCategory::Utility,
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
        }
    }

    /// Returns all available ingredients.
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

    /// Returns the description for this ingredient.
    pub fn description(&self) -> &'static str {
        self.config().description
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
            Ingredient::LapisLazuli => "Mages carved it into wands. The mana practically leaks out.",
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
        }
    }
}

/// A recipe is a combination of ingredients to brew together.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub ingredients: Vec<Ingredient>,
}

impl Recipe {
    /// Creates a new recipe from a list of ingredients.
    pub fn new(ingredients: Vec<Ingredient>) -> Self {
        Self { ingredients }
    }

    /// Time required to brew this recipe (seconds).
    pub fn brew_time(&self) -> f32 {
        BASE_BREW_TIME + PER_INGREDIENT_BREW_TIME * self.ingredients.len() as f32
    }

    /// Duration of the buff after brewing (seconds).
    ///
    /// BuffDurationMultiplier ingredients extend the duration.
    pub fn buff_duration(&self) -> f32 {
        let dilution = self.dilution_factor();
        let mut multiplier = 1.0;
        for ingredient in &self.ingredients {
            if let BrewEffect::BuffDurationMultiplier(v) = ingredient.config().effect {
                multiplier *= 1.0 + (v - 1.0) * dilution;
            }
        }
        BUFF_DURATION * multiplier
    }

    /// Dilution factor — more ingredients means each effect is weaker.
    /// 1 ingredient = 1.0, 2 = ~0.71, 3 = ~0.58, 4 = 0.5
    pub fn dilution_factor(&self) -> f32 {
        if self.ingredients.is_empty() {
            return 0.0;
        }
        1.0 / (self.ingredients.len() as f32).sqrt()
    }

    /// Returns only the base ingredient effects with dilution applied (no combo bonuses).
    pub fn base_effects(&self) -> Vec<BrewEffect> {
        let dilution = self.dilution_factor();
        self.ingredients
            .iter()
            .map(|ingredient| ingredient.config().effect.scale(dilution))
            .collect()
    }

    /// Returns all effects with dilution applied, plus any combo bonuses (undiluted).
    pub fn effects(&self) -> Vec<BrewEffect> {
        let dilution = self.dilution_factor();
        let mut effects: Vec<BrewEffect> = self
            .ingredients
            .iter()
            .map(|ingredient| ingredient.config().effect.scale(dilution))
            .collect();

        // Append combo bonuses (undiluted)
        for combo in self.matching_combos() {
            effects.extend_from_slice(combo.bonus_effects);
        }
        effects
    }

    /// Returns all combos whose required ingredients are all present in this recipe.
    pub fn matching_combos(&self) -> Vec<&'static ComboBonus> {
        COMBOS
            .iter()
            .filter(|combo| {
                combo
                    .required
                    .iter()
                    .all(|req| self.ingredients.contains(req))
            })
            .collect()
    }

    /// Returns the averaged color of all ingredients in the recipe.
    pub fn color(&self) -> Color {
        if self.ingredients.is_empty() {
            return Color::WHITE;
        }
        let count = self.ingredients.len() as f32;
        let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
        for ingredient in &self.ingredients {
            let Srgba {
                red, green, blue, ..
            } = ingredient.config().color.to_srgba();
            r += red;
            g += green;
            b += blue;
        }
        Color::srgb(r / count, g / count, b / count)
    }
}

