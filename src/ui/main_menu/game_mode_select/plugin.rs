use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::escape_to_landing;

use super::systems::{button_action, setup};

/// Plugin that manages the game mode selection screen.
#[derive(Default)]
pub struct GameModeSelectPlugin;

impl Plugin for GameModeSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::GameModeSelect), setup)
            .add_systems(
                OnExit(MenuState::GameModeSelect),
                crate::ui::systems::cleanup_screen::<super::components::OnGameModeSelectScreen>,
            )
            .add_systems(
                Update,
                button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::GameModeSelect)),
            )
            .add_systems(
                Update,
                escape_to_landing.run_if(in_state(MenuState::GameModeSelect)),
            );
    }
}
