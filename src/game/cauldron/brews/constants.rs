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

pub(super) const MISTLETOE_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mistletoe",
    description: "Defenders deal more damage",
    effect: BrewEffect::DefenderDamageBonus(0.5),
    color: Color::srgb(0.78, 0.82, 0.55),
};

pub(super) const VERVAIN_CONFIG: IngredientConfig = IngredientConfig {
    name: "Vervain",
    description: "Increases spell casting speed",
    effect: BrewEffect::CastSpeedMultiplier(1.5),
    color: Color::srgb(0.65, 0.55, 0.72),
};

pub(super) const WORMWOOD_CONFIG: IngredientConfig = IngredientConfig {
    name: "Wormwood",
    description: "Defenders take less damage",
    effect: BrewEffect::DamageResistancePercent(0.25),
    color: Color::srgb(0.68, 0.75, 0.62),
};

pub(super) const BLUE_LOTUS_CONFIG: IngredientConfig = IngredientConfig {
    name: "Blue Lotus",
    description: "Increases spell area of effect",
    effect: BrewEffect::SpellRangeMultiplier(1.4),
    color: Color::srgb(0.52, 0.65, 0.85),
};

pub(super) const MEADOWSWEET_CONFIG: IngredientConfig = IngredientConfig {
    name: "Meadowsweet",
    description: "Defenders move faster",
    effect: BrewEffect::DefenderSpeedBonus(0.3),
    color: Color::srgb(0.92, 0.88, 0.80),
};

pub(super) const VALERIAN_CONFIG: IngredientConfig = IngredientConfig {
    name: "Valerian",
    description: "Slows enemy movement",
    effect: BrewEffect::AttackerSlowPercent(0.25),
    color: Color::srgb(0.42, 0.32, 0.22),
};

pub(super) const NATRON_SALT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Natron Salt",
    description: "Shields defenders with temporary health",
    effect: BrewEffect::DefenderShieldPerSecond(3.0),
    color: Color::srgb(0.92, 0.92, 0.90),
};

pub(super) const BASE_BREW_TIME: f32 = 6.0;
pub(super) const PER_INGREDIENT_BREW_TIME: f32 = 2.0;
pub(super) const BUFF_DURATION: f32 = 30.0;
