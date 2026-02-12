use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

pub struct AchievementPopupPlugin;

impl Plugin for AchievementPopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::spawn_achievement_popup,
                systems::update_achievement_popups,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
