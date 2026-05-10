//! Plugin for version display.

use bevy::prelude::*;

use super::systems;
use crate::state::MenuState;

/// Plugin that displays the clickable version number in the bottom-left corner of the landing screen.
pub struct VersionPlugin;

impl Plugin for VersionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Landing), systems::setup)
            .add_systems(
                OnExit(MenuState::Landing),
                crate::ui::systems::cleanup_screen::<super::components::VersionText>,
            )
            .add_systems(
                Update,
                systems::update_version_link_visuals.run_if(in_state(MenuState::Landing)),
            );
    }
}
