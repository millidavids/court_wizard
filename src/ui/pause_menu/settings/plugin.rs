//! Pause menu settings plugin.
//!
//! This plugin reuses the main menu settings UI systems but with
//! pause menu specific state transitions.

use bevy::prelude::*;

use crate::state::PauseMenuState;
use crate::ui::main_menu::settings::components::SettingsContentContainer;
use crate::ui::main_menu::settings::systems::{
    capture_key_input, handle_confirmation_popup, handle_settings_tab_click,
    key_binding_button_action, key_capture_inactive, option_button_action,
    pause_settings_button_action, rebuild_settings_content, resolution_button_action,
    setup_pause_menu, slider_button_action, slider_interaction, update_key_binding_text,
    update_resolution_selection, update_resolution_visibility, update_selected_options,
    update_slider_text, update_sliders,
};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{escape_to_pause_main, handle_scroll};

/// Plugin that manages the pause menu settings UI.
///
/// Reuses all main menu settings systems except for keyboard input
/// and button actions, which are replaced with pause menu specific versions.
#[derive(Default)]
pub struct PauseSettingsPlugin;

impl Plugin for PauseSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Settings), setup_pause_menu)
            .add_systems(
                OnExit(PauseMenuState::Settings),
                crate::ui::systems::cleanup_screen::<
                    crate::ui::main_menu::settings::components::OnSettingsScreen,
                >,
            )
            .add_systems(
                Update,
                (
                    pause_settings_button_action,
                    option_button_action,
                    slider_button_action,
                    handle_settings_tab_click,
                    key_binding_button_action,
                    resolution_button_action,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(PauseMenuState::Settings)),
            )
            .add_systems(
                Update,
                (
                    escape_to_pause_main.run_if(key_capture_inactive),
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
                    .run_if(in_state(PauseMenuState::Settings)),
            );
    }
}
