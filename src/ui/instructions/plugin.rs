//! Unified instructions screen plugin for both main menu and pause menu.

use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};
use crate::ui::plugin::ButtonActionSet;

use crate::ui::systems::{escape_to_landing, escape_to_pause_main};

use super::components::BackButton;
use super::systems::{cleanup, handle_scroll, setup_main_menu, setup_pause_menu};

/// Plugin that manages the instructions screen UI for the main menu.
#[derive(Default)]
pub struct MainMenuInstructionsPlugin;

impl Plugin for MainMenuInstructionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Instructions), setup_main_menu)
            .add_systems(OnExit(MenuState::Instructions), cleanup)
            .add_systems(
                Update,
                handle_main_menu_back_button
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Instructions)),
            )
            .add_systems(
                Update,
                (handle_scroll, escape_to_landing)
                    .run_if(in_state(MenuState::Instructions)),
            );
    }
}

/// Plugin that manages the instructions screen UI for the pause menu.
#[derive(Default)]
pub struct PauseMenuInstructionsPlugin;

impl Plugin for PauseMenuInstructionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Instructions), setup_pause_menu)
            .add_systems(OnExit(PauseMenuState::Instructions), cleanup)
            .add_systems(
                Update,
                handle_pause_menu_back_button
                    .in_set(ButtonActionSet)
                    .run_if(in_state(PauseMenuState::Instructions)),
            )
            .add_systems(
                Update,
                (handle_scroll, escape_to_pause_main)
                    .run_if(in_state(PauseMenuState::Instructions)),
            );
    }
}

/// Handles back button for main menu instructions.
fn handle_main_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

/// Handles back button for pause menu instructions.
fn handle_pause_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(PauseMenuState::Main);
        }
    }
}
