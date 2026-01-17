//! Pause menu settings plugin.
//!
//! This plugin reuses the main menu settings UI systems but with
//! pause menu specific state transitions.

use bevy::prelude::*;

use crate::state::PauseMenuState;
use crate::ui::main_menu::settings::systems::{
    button_hover, button_press, cleanup, handle_scroll, option_button_action, pause_keyboard_input,
    pause_settings_button_action, setup, slider_button_action, slider_interaction,
    update_selected_options, update_slider_text, update_sliders,
};

/// Plugin that manages the pause menu settings UI.
///
/// Reuses all main menu settings systems except for keyboard input
/// and button actions, which are replaced with pause menu specific versions.
#[derive(Default)]
pub struct PauseSettingsPlugin;

impl Plugin for PauseSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Settings), setup)
            .add_systems(OnExit(PauseMenuState::Settings), cleanup)
            .add_systems(
                Update,
                (
                    pause_keyboard_input,
                    handle_scroll,
                    button_hover,
                    button_press,
                    pause_settings_button_action,
                    option_button_action,
                    slider_button_action,
                    slider_interaction,
                    update_slider_text,
                    update_sliders,
                    update_selected_options,
                )
                    .run_if(in_state(PauseMenuState::Settings)),
            );
    }
}
