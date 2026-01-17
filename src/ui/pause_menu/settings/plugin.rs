//! Pause menu settings plugin.

use bevy::prelude::*;

use crate::state::PauseMenuState;

use super::systems::{
    button_hover, button_press, cleanup, handle_scroll, keyboard_input, option_button_action,
    settings_button_action, setup, slider_button_action, slider_interaction,
    update_selected_options, update_slider_text, update_sliders,
};

/// Plugin that manages the pause menu settings UI.
///
/// Registers systems for:
/// - Pause menu settings setup and cleanup
/// - Keyboard input handling
/// - Button interaction and actions
/// - Unified slider controls for all config values
/// - Selected option highlighting
#[derive(Default)]
pub struct PauseSettingsPlugin;

impl Plugin for PauseSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseMenuState::Settings), setup)
            .add_systems(OnExit(PauseMenuState::Settings), cleanup)
            .add_systems(
                Update,
                (
                    keyboard_input,
                    handle_scroll,
                    button_hover,
                    button_press,
                    settings_button_action,
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
