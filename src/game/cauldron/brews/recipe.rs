use bevy::prelude::*;

use super::combo::ComboBonus;
use super::constants::*;
use super::effect::BrewEffect;
use super::ingredient::Ingredient;

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
    /// If Philosopher's Stone is present, dilution is removed (returns 1.0).
    pub fn dilution_factor(&self) -> f32 {
        if self.ingredients.is_empty() {
            return 0.0;
        }
        if self.ingredients.iter().any(|i| i.is_philosophers_stone()) {
            return 1.0;
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
