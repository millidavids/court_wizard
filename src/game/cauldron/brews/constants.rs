use bevy::prelude::*;

use super::{BrewEffect, ComboBonus, Ingredient, IngredientConfig};

pub(super) const LAVENDER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Lavender",
    flavor_text: "A fragrant purple shrub that carpets sun-baked hillsides and dry meadows. Its slender flower spikes release a calming scent prized by healers and mages alike.",
    functional_description: "Increases mana regeneration.",
    effect: BrewEffect::ManaRegenMultiplier(2.0),
    color: Color::srgb(0.6, 0.4, 0.8),
};

pub(super) const MUGWORT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mugwort",
    flavor_text: "A tall, silvery-leafed weed that thrives along roadsides and riverbanks. Its downy underside glows faintly in moonlight, hinting at its arcane potency.",
    functional_description: "Increases spell power.",
    effect: BrewEffect::SpellPowerMultiplier(1.5),
    color: Color::srgb(0.3, 0.5, 0.2),
};

pub(super) const YARROW_CONFIG: IngredientConfig = IngredientConfig {
    name: "Yarrow",
    flavor_text: "A hardy wildflower with flat clusters of tiny white blossoms and feathery leaves. It grows in open grasslands and along dusty field edges.",
    functional_description: "Heals defender units over time.",
    effect: BrewEffect::DefenderHealPerSecond(5.0),
    color: Color::srgb(0.95, 0.9, 0.75),
};

pub(super) const MISTLETOE_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mistletoe",
    flavor_text: "A parasitic evergreen found tangled high in oak branches, bearing pale green leaves and waxy white berries. Druids harvested it with golden sickles at midwinter.",
    functional_description: "Defenders deal more damage.",
    effect: BrewEffect::DefenderDamageBonus(0.5),
    color: Color::srgb(0.78, 0.82, 0.55),
};

pub(super) const VERVAIN_CONFIG: IngredientConfig = IngredientConfig {
    name: "Vervain",
    flavor_text: "A slender herb with small lilac flowers that grows in dry ditches and along crumbling stone walls. Long considered sacred by enchanters for its ability to quicken rituals.",
    functional_description: "Increases spell casting speed.",
    effect: BrewEffect::CastSpeedMultiplier(1.5),
    color: Color::srgb(0.65, 0.55, 0.72),
};

pub(super) const WORMWOOD_CONFIG: IngredientConfig = IngredientConfig {
    name: "Wormwood",
    flavor_text: "A bitter, grey-green shrub with deeply lobed leaves that flourishes in rocky wastelands and abandoned ruins. Its pungent oil wards off both insects and blades.",
    functional_description: "Defenders take less damage.",
    effect: BrewEffect::DamageResistancePercent(0.25),
    color: Color::srgb(0.68, 0.75, 0.62),
};

pub(super) const BLUE_LOTUS_CONFIG: IngredientConfig = IngredientConfig {
    name: "Blue Lotus",
    flavor_text: "A striking water flower with vivid blue petals that floats on still temple pools and warm river deltas. Its intoxicating fragrance expands the mind — and spell radius.",
    functional_description: "Increases spell area of effect.",
    effect: BrewEffect::SpellRangeMultiplier(1.4),
    color: Color::srgb(0.52, 0.65, 0.85),
};

pub(super) const MEADOWSWEET_CONFIG: IngredientConfig = IngredientConfig {
    name: "Meadowsweet",
    flavor_text: "A frothy-topped plant with creamy white blossoms that sways in damp meadows and along stream banks. Its sweet, almond-like scent invigorates those who breathe it in.",
    functional_description: "Defenders move faster.",
    effect: BrewEffect::DefenderSpeedBonus(0.3),
    color: Color::srgb(0.92, 0.88, 0.80),
};

pub(super) const VALERIAN_CONFIG: IngredientConfig = IngredientConfig {
    name: "Valerian",
    flavor_text: "A tall plant with dense pink flower heads and a thick, pungent root found in wet woodland clearings. Its earthy smell is overwhelming — and deeply sedating.",
    functional_description: "Slows enemy movement.",
    effect: BrewEffect::AttackerSlowPercent(0.25),
    color: Color::srgb(0.42, 0.32, 0.22),
};

pub(super) const NATRON_SALT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Natron Salt",
    flavor_text: "A chalky white mineral scraped from the shores of dried desert lakes. Ancient embalmers used it to preserve the dead; alchemists use it to preserve the living.",
    functional_description: "Shields defenders with temporary health.",
    effect: BrewEffect::DefenderShieldPerSecond(3.0),
    color: Color::srgb(0.92, 0.92, 0.90),
};

// ===== New Ingredient Configs =====

pub(super) const LAPIS_LAZULI_CONFIG: IngredientConfig = IngredientConfig {
    name: "Lapis Lazuli",
    flavor_text: "A deep blue gemstone flecked with golden pyrite, mined from mountain caves in arid highlands. Mages have carved it into wands and amulets for centuries.",
    functional_description: "Increases maximum mana capacity.",
    effect: BrewEffect::MaxManaMultiplier(1.5),
    color: Color::srgb(0.18, 0.28, 0.52),
};

pub(super) const HENBANE_CONFIG: IngredientConfig = IngredientConfig {
    name: "Henbane",
    flavor_text: "A sinister-looking plant with sticky, hairy stems and pale yellow flowers veined in purple. It sprouts in graveyards and disturbed soil.",
    functional_description: "Defenders attack faster.",
    effect: BrewEffect::AttackSpeedMultiplier(1.4),
    color: Color::srgb(0.18, 0.12, 0.08),
};

pub(super) const FRANKINCENSE_CONFIG: IngredientConfig = IngredientConfig {
    name: "Frankincense",
    flavor_text: "A golden, tear-shaped resin that oozes from the bark of gnarled trees clinging to desert cliffs. When burned, its thick smoke carries spells further and harder.",
    functional_description: "Increases spell power.",
    effect: BrewEffect::SpellPowerMultiplier(1.3),
    color: Color::srgb(0.88, 0.75, 0.48),
};

pub(super) const AMBER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Amber",
    flavor_text: "A warm, translucent gemstone of fossilized tree resin, often found washed ashore on cold northern beaches or buried in ancient pine forests. It traps time itself within its glow.",
    functional_description: "Brew buffs last longer.",
    effect: BrewEffect::BuffDurationMultiplier(1.5),
    color: Color::srgb(0.85, 0.60, 0.28),
};

pub(super) const RAVEN_FEATHER_CONFIG: IngredientConfig = IngredientConfig {
    name: "Raven Feather",
    flavor_text: "An iridescent black feather with a faint blue-violet sheen, plucked from ravens that roost on ancient battlefields and gallows hills. It hums with gathered wisdom.",
    functional_description: "Boosts defender effectiveness.",
    effect: BrewEffect::EffectivenessBonus(0.3),
    color: Color::srgb(0.08, 0.08, 0.12),
};

pub(super) const MANDRAKE_ROOT_CONFIG: IngredientConfig = IngredientConfig {
    name: "Mandrake Root",
    flavor_text: "A forked, pale root with an unsettling resemblance to a human figure, dug from rich soil beneath gallows trees. Harvesters stuff their ears with wax before pulling it.",
    functional_description: "Increases spell casting speed.",
    effect: BrewEffect::CastSpeedMultiplier(1.25),
    color: Color::srgb(0.35, 0.25, 0.18),
};

pub(super) const ROWAN_BERRY_CONFIG: IngredientConfig = IngredientConfig {
    name: "Rowan Berry",
    flavor_text: "Bright red-orange berries that grow in dense clusters on mountain rowan trees. Found at forest edges and highland passes, they have long been strung above doorways to repel evil.",
    functional_description: "Defenders take less damage.",
    effect: BrewEffect::DamageResistancePercent(0.2),
    color: Color::srgb(0.92, 0.32, 0.18),
};

pub(super) const DRAGONS_BLOOD_CONFIG: IngredientConfig = IngredientConfig {
    name: "Dragon's Blood",
    flavor_text: "A dark crimson resin harvested from thorny tropical trees that grow in volcanic soil. It bleeds from cuts in the bark and dries into brittle, blood-red flakes.",
    functional_description: "Defenders deal more damage.",
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
