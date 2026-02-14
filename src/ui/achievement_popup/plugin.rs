use bevy::prelude::*;

use super::components::AchievementQueue;
use super::systems;

pub struct AchievementPopupPlugin;

impl Plugin for AchievementPopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AchievementQueue>().add_systems(
            Update,
            (
                systems::queue_achievements,
                systems::spawn_next_popup,
                systems::update_achievement_popups,
            )
                .chain(),
        );
    }
}
