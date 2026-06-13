//! Settings menu plugin.

use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;

use super::components::{
    KeyCaptureState, SettingsContentContainer, SettingsTabState, SliderAdjusted,
};
use crate::ui::systems::{escape_to_landing, handle_scroll};

use super::systems::{
    capture_key_input, cycle_controller_diagram_scheme, handle_confirmation_popup,
    handle_settings_tab_click, key_binding_button_action, key_capture_inactive,
    option_button_action, rebuild_settings_content, resolution_button_action,
    settings_button_action, setup_main_menu, slider_button_action, slider_interaction,
    update_key_binding_text, update_resolution_selection, update_resolution_visibility,
    update_selected_options, update_slider_text, update_sliders,
};

/// Plugin that manages the settings menu UI.
///
/// Registers systems for:
/// - Settings menu setup and cleanup
/// - Keyboard input handling
/// - Button interaction and actions
/// - Unified slider controls for all config values
/// - Selected option highlighting
/// - Tab switching and content rebuilding
#[derive(Default)]
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsTabState>()
            .init_resource::<KeyCaptureState>()
            .add_message::<SliderAdjusted>()
            .add_systems(OnEnter(MenuState::Settings), setup_main_menu)
            .add_systems(
                OnExit(MenuState::Settings),
                crate::ui::systems::cleanup_screen::<super::components::OnSettingsScreen>,
            )
            .add_systems(
                Update,
                (
                    settings_button_action,
                    option_button_action,
                    slider_button_action,
                    handle_settings_tab_click,
                    key_binding_button_action,
                    resolution_button_action,
                )
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::Settings)),
            )
            .add_systems(
                Update,
                (
                    escape_to_landing.run_if(key_capture_inactive),
                    handle_scroll::<SettingsContentContainer>,
                    slider_interaction,
                    update_slider_text,
                    update_sliders,
                    update_selected_options,
                    handle_confirmation_popup,
                    rebuild_settings_content,
                    cycle_controller_diagram_scheme,
                    capture_key_input,
                    update_key_binding_text,
                    update_resolution_selection,
                    update_resolution_visibility,
                )
                    .run_if(in_state(MenuState::Settings)),
            );

        // Dev-only: orange "Unlock Everything" button click handler + F2 reveal.
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            super::debug_unlock::unlock_everything_button_action
                .in_set(ButtonActionSet)
                .run_if(in_state(MenuState::Settings)),
        )
        .add_systems(
            Update,
            crate::game::debug_ui::sync_marker_visibility::<
                super::debug_unlock::UnlockEverythingButton,
            >
                .run_if(in_state(MenuState::Settings)),
        );
    }
}
