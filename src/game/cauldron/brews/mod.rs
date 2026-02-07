/// A single effect that an ingredient/brew applies when active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrewEffect {
    /// Multiplies wizard mana regeneration rate.
    ManaRegenMultiplier(f32),
    /// Multiplies spell power/empowerment.
    SpellPowerMultiplier(f32),
}

/// An ingredient that can be added to a brew.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ingredient {
    Lavender,
    Mugwort,
}

/// Static configuration for an ingredient.
pub struct IngredientConfig {
    /// Display name shown in the UI.
    pub name: &'static str,
    /// Short description of the ingredient's effect.
    pub description: &'static str,
    /// The effect this ingredient contributes at full strength.
    pub effect: BrewEffect,
}

const LAVENDER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Lavender",
    description: "Increases mana regeneration",
    effect: BrewEffect::ManaRegenMultiplier(2.0),
};

const MUGWORT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mugwort",
    description: "Increases spell power",
    effect: BrewEffect::SpellPowerMultiplier(1.5),
};

impl Ingredient {
    /// Returns the static configuration for this ingredient.
    pub fn config(&self) -> &'static IngredientConfig {
        match self {
            Ingredient::Lavender => &LAVENDER_CONFIG,
            Ingredient::Mugwort => &MUGWORT_CONFIG,
        }
    }

    /// Returns all available ingredients.
    pub const fn all() -> &'static [Ingredient] {
        &[Ingredient::Lavender, Ingredient::Mugwort]
    }

    /// Returns the display name for this ingredient.
    pub fn name(&self) -> &'static str {
        self.config().name
    }

    /// Returns the description for this ingredient.
    pub fn description(&self) -> &'static str {
        self.config().description
    }
}

/// A recipe is a combination of ingredients to brew together.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub ingredients: Vec<Ingredient>,
}

const BASE_BREW_TIME: f32 = 6.0;
const PER_INGREDIENT_BREW_TIME: f32 = 2.0;
const BUFF_DURATION: f32 = 30.0;

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
    pub fn buff_duration(&self) -> f32 {
        BUFF_DURATION
    }

    /// Dilution factor — more ingredients means each effect is weaker.
    /// 1 ingredient = 1.0, 2 = ~0.71, 3 = ~0.58, 4 = 0.5
    pub fn dilution_factor(&self) -> f32 {
        if self.ingredients.is_empty() {
            return 0.0;
        }
        1.0 / (self.ingredients.len() as f32).sqrt()
    }

    /// Returns all effects with dilution applied.
    pub fn effects(&self) -> Vec<BrewEffect> {
        let dilution = self.dilution_factor();
        self.ingredients
            .iter()
            .map(|ingredient| {
                let base_effect = ingredient.config().effect;
                dilute_effect(base_effect, dilution)
            })
            .collect()
    }
}

/// Applies dilution to an effect's magnitude.
/// Formula: effective = 1.0 + (base - 1.0) * dilution
fn dilute_effect(effect: BrewEffect, dilution: f32) -> BrewEffect {
    match effect {
        BrewEffect::ManaRegenMultiplier(v) => {
            BrewEffect::ManaRegenMultiplier(1.0 + (v - 1.0) * dilution)
        }
        BrewEffect::SpellPowerMultiplier(v) => {
            BrewEffect::SpellPowerMultiplier(1.0 + (v - 1.0) * dilution)
        }
    }
}
