//! Plugin for changelog screen.

use bevy::prelude::*;

use super::systems;
use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;

/// Plugin that handles the changelog screen.
pub struct ChangelogPlugin;

impl Plugin for ChangelogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Changelog), systems::setup)
            .add_systems(
                Update,
                systems::handle_back_button
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Changelog)),
            )
            .add_systems(
                Update,
                (systems::update_button_colors, systems::handle_scroll)
                    .run_if(in_state(MenuState::Changelog)),
            )
            .add_systems(OnExit(MenuState::Changelog), systems::cleanup);
    }
}
