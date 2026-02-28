//! Progress screen plugin for both main menu and pause menu.

use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use crate::game::achievements::messages::ClearProgressMessage;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};
use crate::ui::plugin::ButtonActionSet;

use super::components::{
    BackButton, CancelClearButton, ClearProgressButton, ConfirmClearButton, ConfirmationPopup,
};
use crate::ui::systems::{escape_to_landing, escape_to_pause_main};

use super::systems::{
    cleanup, clear_and_refresh_main_menu, clear_and_refresh_pause_menu, handle_clear_progress,
    handle_scroll, setup_main_menu, setup_pause_menu, spawn_confirmation_popup,
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
                    handle_main_menu_confirm_clear,
                    handle_main_menu_cancel_clear,
                    update_button_colors,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Progress)),
            )
            .add_systems(
                Update,
                (handle_scroll, escape_to_landing)
                    .run_if(in_state(MenuState::Progress)),
            )
            .add_systems(
                Update,
                clear_and_refresh_main_menu
                    .run_if(on_message::<ClearProgressMessage>)
                    .run_if(in_state(MenuState::Progress)),
            );
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
                    handle_pause_menu_confirm_clear,
                    handle_pause_menu_cancel_clear,
                    update_button_colors,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(PauseMenuState::Progress)),
            )
            .add_systems(
                Update,
                (handle_scroll, escape_to_pause_main)
                    .run_if(in_state(PauseMenuState::Progress)),
            )
            .add_systems(
                Update,
                clear_and_refresh_pause_menu
                    .run_if(on_message::<ClearProgressMessage>)
                    .run_if(in_state(PauseMenuState::Progress)),
            );
    }
}

/// Handles back button for main menu progress.
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

/// Handles clear progress button from main menu — shows confirmation popup.
fn handle_main_menu_clear_progress(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ClearProgressButton>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            spawn_confirmation_popup(&mut commands);
        }
    }
}

/// Handles confirm button in popup from main menu — clears save data and writes message.
fn handle_main_menu_confirm_clear(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ConfirmClearButton>,
    popup_query: Query<Entity, With<ConfirmationPopup>>,
    mut clear_msg: MessageWriter<ClearProgressMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            handle_clear_progress();
            clear_msg.write(ClearProgressMessage);
            // Despawn popup
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handles cancel button in popup from main menu — just closes the popup.
fn handle_main_menu_cancel_clear(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&CancelClearButton>,
    popup_query: Query<Entity, With<ConfirmationPopup>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            // Despawn popup
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handles clear progress button from pause menu — shows confirmation popup.
fn handle_pause_menu_clear_progress(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ClearProgressButton>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            spawn_confirmation_popup(&mut commands);
        }
    }
}

/// Handles confirm button in popup from pause menu — clears save data and writes message.
fn handle_pause_menu_confirm_clear(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ConfirmClearButton>,
    popup_query: Query<Entity, With<ConfirmationPopup>>,
    mut clear_msg: MessageWriter<ClearProgressMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            handle_clear_progress();
            clear_msg.write(ClearProgressMessage);
            // Despawn popup
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handles cancel button in popup from pause menu — just closes the popup.
fn handle_pause_menu_cancel_clear(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&CancelClearButton>,
    popup_query: Query<Entity, With<ConfirmationPopup>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            // Despawn popup
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
        }
    }
}
