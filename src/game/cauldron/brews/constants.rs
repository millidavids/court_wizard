use bevy::prelude::*;

use super::{BrewEffect, IngredientConfig};

pub(super) const LAVENDER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Lavender",
    description: "Increases mana regeneration",
    effect: BrewEffect::ManaRegenMultiplier(2.0),
    color: Color::srgb(0.6, 0.4, 0.8),
};

pub(super) const MUGWORT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mugwort",
    description: "Increases spell power",
    effect: BrewEffect::SpellPowerMultiplier(1.5),
    color: Color::srgb(0.3, 0.5, 0.2),
};

pub(super) const YARROW_CONFIG: IngredientConfig = IngredientConfig {
    name: "Yarrow",
    description: "Heals defender units over time",
    effect: BrewEffect::DefenderHealPerSecond(5.0),
    color: Color::srgb(0.95, 0.9, 0.75),
};

pub(super) const BASE_BREW_TIME: f32 = 6.0;
pub(super) const PER_INGREDIENT_BREW_TIME: f32 = 2.0;
pub(super) const BUFF_DURATION: f32 = 30.0;
