use super::effect::BrewEffect;
use super::ingredient::Ingredient;

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
