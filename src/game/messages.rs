use bevy::prelude::*;

use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;

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
