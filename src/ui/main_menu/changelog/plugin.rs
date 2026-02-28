//! Plugin for changelog screen.

use bevy::prelude::*;

use super::components::ScrollableChangelogContainer;
use super::systems;
use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{escape_to_landing, handle_scroll};

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
                (handle_scroll::<ScrollableChangelogContainer>, escape_to_landing)
                    .run_if(in_state(MenuState::Changelog)),
            )
            .add_systems(OnExit(MenuState::Changelog), systems::cleanup);
    }
}
