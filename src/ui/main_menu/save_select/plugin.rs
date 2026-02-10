//! Save select screen plugin.

use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that manages the save select screen UI.
#[derive(Default)]
pub struct SaveSelectPlugin;

impl Plugin for SaveSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::SaveSelect), systems::setup)
            .add_systems(OnExit(MenuState::SaveSelect), systems::cleanup)
            .add_systems(
                Update,
                systems::button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::SaveSelect)),
            )
            .add_systems(
                Update,
                systems::keyboard_input.run_if(in_state(MenuState::SaveSelect)),
            );
    }
}
