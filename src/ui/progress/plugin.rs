//! Progress screen plugin for both main menu and pause menu.

use bevy::prelude::*;

use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};
use crate::ui::plugin::ButtonActionSet;

use super::components::{BackButton, ClearProgressButton};
use super::systems::{
    cleanup, handle_clear_progress, handle_scroll, setup_main_menu, setup_pause_menu,
    update_button_colors,
};

/// Plugin that manages the progress screen UI for the main menu.
#[derive(Default)]
pub struct MainMenuProgressPlugin;

impl Plugin for MainMenuProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Progress), setup_main_menu)
            .add_systems(OnExit(MenuState::Progress), cleanup)
            .add_systems(
                Update,
                (
                    handle_main_menu_back_button,
                    handle_main_menu_clear_progress,
                    update_button_colors,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Progress)),
            )
            .add_systems(Update, handle_scroll.run_if(in_state(MenuState::Progress)));
    }
}

/// Plugin that manages the progress screen UI for the pause menu.
#[derive(Default)]
pub struct PauseMenuProgressPlugin;

impl Plugin for PauseMenuProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Progress), setup_pause_menu)
            .add_systems(OnExit(PauseMenuState::Progress), cleanup)
            .add_systems(
                Update,
                (
                    handle_pause_menu_back_button,
                    handle_pause_menu_clear_progress,
                    update_button_colors,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(PauseMenuState::Progress)),
            )
            .add_systems(
                Update,
                handle_scroll.run_if(in_state(PauseMenuState::Progress)),
            );
    }
}

/// Handles back button for main menu progress.
fn handle_main_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(MenuState::Landing);
        }
    }
}

/// Handles back button for pause menu progress.
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

/// Handles clear progress button from main menu — clears data and refreshes screen.
fn handle_main_menu_clear_progress(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ClearProgressButton>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            handle_clear_progress();
            // Re-enter same state to refresh UI with cleared data
            next_state.set(MenuState::Progress);
        }
    }
}

/// Handles clear progress button from pause menu — clears data and refreshes screen.
fn handle_pause_menu_clear_progress(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ClearProgressButton>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            handle_clear_progress();
            // Re-enter same state to refresh UI with cleared data
            next_state.set(PauseMenuState::Progress);
        }
    }
}
