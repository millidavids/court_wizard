//! Multiplayer pause-menu settings plugin.
//!
//! Mirrors [`super::settings::plugin::PauseSettingsPlugin`] but binds to
//! [`MultiplayerGameState::Settings`] and returns to
//! [`MultiplayerGameState::Paused`] on Back instead of `PauseMenuState::Main`.
//! Reuses every UI builder and interaction system from the main-menu settings
//! module — only the Back action differs.

use bevy::prelude::*;

use crate::game::input::messages::MouseClicked;
use crate::state::MultiplayerGameState;
use crate::ui::main_menu::settings::components::{
    ConfirmationPopup, SettingsButtonAction, SettingsContentContainer,
};
use crate::ui::main_menu::settings::systems::{
    capture_key_input, handle_confirmation_popup, handle_settings_tab_click,
    key_binding_button_action, key_capture_inactive, option_button_action,
    rebuild_settings_content, resolution_button_action, setup_pause_menu, slider_button_action,
    slider_interaction, spawn_confirmation_popup, update_key_binding_text,
    update_resolution_selection, update_resolution_visibility, update_selected_options,
    update_slider_text, update_sliders,
};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::handle_scroll;

/// Plugin that manages the multiplayer pause-menu settings UI.
#[derive(Default)]
pub struct MpPauseSettingsPlugin;

impl Plugin for MpPauseSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MultiplayerGameState::Settings), setup_pause_menu)
            .add_systems(
                OnExit(MultiplayerGameState::Settings),
                crate::ui::systems::cleanup_screen::<
                    crate::ui::main_menu::settings::components::OnSettingsScreen,
                >,
            )
            .add_systems(
                Update,
                (
                    mp_pause_settings_button_action,
                    option_button_action,
                    slider_button_action,
                    handle_settings_tab_click,
                    key_binding_button_action,
                    resolution_button_action,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MultiplayerGameState::Settings)),
            )
            .add_systems(
                Update,
                (
                    mp_escape_to_paused.run_if(key_capture_inactive),
                    handle_scroll::<SettingsContentContainer>,
                    slider_interaction,
                    update_slider_text,
                    update_sliders,
                    update_selected_options,
                    handle_confirmation_popup,
                    rebuild_settings_content,
                    capture_key_input,
                    update_key_binding_text,
                    update_resolution_selection,
                    update_resolution_visibility,
                )
                    .run_if(in_state(MultiplayerGameState::Settings)),
            );

        // Dev-only: orange "Unlock Everything" button click handler + F2 reveal.
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            crate::ui::main_menu::settings::debug_unlock::unlock_everything_button_action
                .in_set(ButtonActionSet)
                .run_if(in_state(MultiplayerGameState::Settings)),
        )
        .add_systems(
            Update,
            crate::game::debug_ui::sync_marker_visibility::<
                crate::ui::main_menu::settings::debug_unlock::UnlockEverythingButton,
            >
                .run_if(in_state(MultiplayerGameState::Settings)),
        );
    }
}

/// Mirrors `pause_settings_button_action` but returns to
/// `MultiplayerGameState::Paused` on Back instead of `PauseMenuState::Main`.
fn mp_pause_settings_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SettingsButtonAction>,
    popup_query: Query<&ConfirmationPopup>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    mut bindings: ResMut<crate::config::InputBindings>,
) {
    if !popup_query.is_empty() {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                SettingsButtonAction::Back => {
                    next_mp_state.set(MultiplayerGameState::Paused);
                }
                SettingsButtonAction::ResetTutorials => {
                    spawn_confirmation_popup(&mut commands, *action, "Reset all tutorials?");
                }
                SettingsButtonAction::ClearProgress => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Clear all progress? This cannot be undone.",
                    );
                }
                SettingsButtonAction::ResetControls => {
                    *bindings = crate::config::InputBindings::default();
                }
            }
        }
    }
}

/// Escape key handler while in `MultiplayerGameState::Settings` — returns to
/// the MP pause menu. Suppressed while a key-capture binding is active so the
/// player can bind Escape to an action.
fn mp_escape_to_paused(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_mp_state.set(MultiplayerGameState::Paused);
    }
}
