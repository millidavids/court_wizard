use bevy::prelude::*;

use super::systems;

pub struct AchievementPopupPlugin;

impl Plugin for AchievementPopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::spawn_achievement_popup,
                systems::update_achievement_popups,
            ),
        );
    }
}
