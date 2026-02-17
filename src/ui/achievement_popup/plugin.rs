use bevy::prelude::*;

use super::components::PopupQueue;
use super::systems;

pub struct AchievementPopupPlugin;

impl Plugin for AchievementPopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PopupQueue>().add_systems(
            Update,
            (
                systems::queue_popups,
                systems::spawn_next_popup,
                systems::update_achievement_popups,
            )
                .chain(),
        );
    }
}
