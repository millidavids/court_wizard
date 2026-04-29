use bevy::prelude::*;

use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::wizard::components::Spell;

/// Message sent when an achievement is unlocked during gameplay.
#[derive(Message)]
pub(crate) struct AchievementUnlockedMessage {
    pub(crate) id: AchievementId,
}

/// Message sent when an ingredient is collected via Telekinesis during gameplay.
#[derive(Message)]
pub(crate) struct IngredientCollectedMessage {
    pub(crate) ingredient: Ingredient,
}

/// Message sent when a spell is fully researched on the Wizard Tower screen.
#[derive(Message)]
pub(crate) struct SpellResearchedMessage {
    pub(crate) spell: Spell,
}

/// Message sent when a new wave of attackers spawns during gameplay.
#[derive(Message)]
pub(crate) struct WaveSpawnedMessage {
    /// The wave number (1-indexed for display).
    pub(crate) wave_number: u32,
}

/// Message sent when the King calls for a retreat.
#[derive(Message)]
pub(crate) struct RetreatMessage;

/// Message sent when insight bonus levels are upgraded via the study tab.
#[derive(Message)]
pub(crate) struct InsightBonusUpgradedMessage;

/// Message sent when a hidden ingredient combo is discovered during brewing.
#[derive(Message)]
pub(crate) struct ComboDiscoveredMessage {
    /// Display name of the combo.
    pub(crate) name: &'static str,
    /// Flavor text description.
    pub(crate) description: &'static str,
}

/// Message sent when a spell's talent progress crosses a tier threshold.
#[derive(Message)]
pub(crate) struct TalentTierUnlockedMessage {
    pub(crate) spell: Spell,
    pub(crate) tier: u8,
}
