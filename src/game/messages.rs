use bevy::prelude::*;

use crate::config::save_data::AchievementId;

/// Message sent when an achievement is unlocked during gameplay.
#[derive(Message)]
pub(crate) struct AchievementUnlockedMessage {
    pub(crate) id: AchievementId,
}
