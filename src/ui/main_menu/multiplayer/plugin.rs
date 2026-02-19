//! Multiplayer screen plugin.

use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;

use super::systems::{button_action, cleanup, process_lobby_messages, setup, update_ui_state};

/// Plugin that manages the multiplayer lobby screen UI.
#[derive(Default)]
pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Multiplayer), setup)
            .add_systems(OnExit(MenuState::Multiplayer), cleanup)
            .add_systems(
                Update,
                (
                    button_action.in_set(ButtonActionSet),
                    process_lobby_messages,
                    update_ui_state,
                )
                    .chain()
                    .run_if(in_state(MenuState::Multiplayer)),
            );
    }
}
