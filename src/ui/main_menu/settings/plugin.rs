//! Settings menu plugin.

use bevy::prelude::*;

use crate::state::MenuState;

use super::systems::{
    button_action, button_hover, button_press, cleanup, handle_scroll, keyboard_input, setup,
    ui_brightness_button_action, update_selected_options, update_ui_brightness_text,
    update_volume_sliders, update_volume_text, volume_button_action, volume_slider_interaction,
};

/// Plugin that manages the settings menu UI.
///
/// Registers systems for:
/// - Settings menu setup and cleanup
/// - Keyboard input handling
/// - Button interaction and actions
/// - Volume control updates
/// - UI brightness control updates
/// - Selected option highlighting
#[derive(Default)]
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Settings), setup)
            .add_systems(OnExit(MenuState::Settings), cleanup)
            .add_systems(
                Update,
                (
                    keyboard_input,
                    handle_scroll,
                    button_hover,
                    button_press,
                    button_action,
                    volume_button_action,
                    volume_slider_interaction,
                    ui_brightness_button_action,
                    update_volume_text,
                    update_volume_sliders,
                    update_ui_brightness_text,
                    update_selected_options,
                )
                    .run_if(in_state(MenuState::Settings)),
            );
    }
}
