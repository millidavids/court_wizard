use bevy::prelude::*;

use super::{BrewEffect, ComboBonus, Ingredient, IngredientConfig};

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

// ===== New Ingredient Configs =====

pub(super) const LAPIS_LAZULI_CONFIG: IngredientConfig = IngredientConfig {
    name: "Lapis Lazuli",
    description: "Increases maximum mana capacity",
    effect: BrewEffect::MaxManaMultiplier(1.5),
    color: Color::srgb(0.18, 0.28, 0.52),
};

pub(super) const HENBANE_CONFIG: IngredientConfig = IngredientConfig {
    name: "Henbane",
    description: "Defenders attack faster",
    effect: BrewEffect::AttackSpeedMultiplier(1.4),
    color: Color::srgb(0.18, 0.12, 0.08),
};

pub(super) const FRANKINCENSE_CONFIG: IngredientConfig = IngredientConfig {
    name: "Frankincense",
    description: "Increases spell power",
    effect: BrewEffect::SpellPowerMultiplier(1.3),
    color: Color::srgb(0.88, 0.75, 0.48),
};

pub(super) const AMBER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Amber",
    description: "Brew buffs last longer",
    effect: BrewEffect::BuffDurationMultiplier(1.5),
    color: Color::srgb(0.85, 0.60, 0.28),
};

pub(super) const RAVEN_FEATHER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Raven Feather",
    description: "Boosts defender effectiveness",
    effect: BrewEffect::EffectivenessBonus(0.3),
    color: Color::srgb(0.08, 0.08, 0.12),
};

pub(super) const MANDRAKE_ROOT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mandrake Root",
    description: "Increases spell casting speed",
    effect: BrewEffect::CastSpeedMultiplier(1.25),
    color: Color::srgb(0.35, 0.25, 0.18),
};

pub(super) const ROWAN_BERRY_CONFIG: IngredientConfig = IngredientConfig {
    name: "Rowan Berry",
    description: "Defenders take less damage",
    effect: BrewEffect::DamageResistancePercent(0.2),
    color: Color::srgb(0.92, 0.32, 0.18),
};

pub(super) const DRAGONS_BLOOD_CONFIG: IngredientConfig = IngredientConfig {
    name: "Dragon's Blood",
    description: "Defenders deal more damage",
    effect: BrewEffect::DefenderDamageBonus(0.35),
    color: Color::srgb(0.65, 0.15, 0.12),
};

// ===== Brew Timing =====

pub(super) const BASE_BREW_TIME: f32 = 6.0;
pub(super) const PER_INGREDIENT_BREW_TIME: f32 = 2.0;
pub(super) const BUFF_DURATION: f32 = 30.0;

// ===== Hidden Combos =====

pub(super) const COMBOS: &[ComboBonus] = &[
    ComboBonus {
        name: "War Brew",
        required: &[Ingredient::Mistletoe, Ingredient::Henbane],
        bonus_effects: &[BrewEffect::DefenderDamageBonus(0.15)],
        description: "Druidic weapon blessing meets berserker fury",
    },
    ComboBonus {
        name: "Arcane Surge",
        required: &[Ingredient::Lavender, Ingredient::LapisLazuli],
        bonus_effects: &[BrewEffect::ManaRegenMultiplier(1.15)],
        description: "Mana regen and mana pool synergy",
    },
    ComboBonus {
        name: "Warding Circle",
        required: &[Ingredient::Wormwood, Ingredient::RowanBerry],
        bonus_effects: &[BrewEffect::DamageResistancePercent(0.1)],
        description: "Double ward grants total protection",
    },
    ComboBonus {
        name: "Masterwork Elixir",
        required: &[
            Ingredient::Mugwort,
            Ingredient::Frankincense,
            Ingredient::BlueLotus,
        ],
        bonus_effects: &[BrewEffect::SpellRangeMultiplier(1.1)],
        description: "Triple spell-enhancers amplify range",
    },
    ComboBonus {
        name: "Enduring Brew",
        required: &[Ingredient::Amber, Ingredient::Yarrow],
        bonus_effects: &[BrewEffect::DefenderHealPerSecond(1.5)],
        description: "Preservation meets healing",
    },
    ComboBonus {
        name: "Berserker's Draught",
        required: &[Ingredient::Henbane, Ingredient::DragonsBlood],
        bonus_effects: &[BrewEffect::AttackSpeedMultiplier(1.1)],
        description: "Rage herb empowered by wyrm blood",
    },
    ComboBonus {
        name: "Wizard's Focus",
        required: &[Ingredient::LapisLazuli, Ingredient::MandrakeRoot],
        bonus_effects: &[BrewEffect::CastSpeedMultiplier(1.1)],
        description: "Deep mana meets spirit binding",
    },
];
