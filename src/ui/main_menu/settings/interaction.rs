//! Settings interaction handlers and confirmation popup logic.

use super::builders::spawn_confirmation_popup;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::GameConfig;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};

use super::components::{
    ButtonColors, ConfirmationAction, ConfirmationPopup, OptionButtonValue, SelectedOption,
    SettingsButtonAction, SliderAdjusted, SliderDownButton, SliderFill, SliderHandle, SliderText,
    SliderTrack, SliderUpButton,
};
use super::constants::{BUTTON_BACKGROUND, BUTTON_BORDER, SELECTED_BACKGROUND, SELECTED_BORDER};

/// Sets up the settings menu UI with a tabbed interface.
///
/// Creates a settings screen with tabs for Graphics, Audio, Game, and Controls.
/// Tab content is rebuilt dynamically by `rebuild_settings_content` when the
/// active tab changes.
///
/// All spawned entities are marked with `OnSettingsScreen` for cleanup.
#[allow(clippy::too_many_arguments)]
pub fn handle_confirmation_popup(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    mut back_msgs: MessageReader<crate::game::input::gamepad::messages::MenuBackPressed>,
    action_query: Query<&ConfirmationAction>,
    popup_query: Query<Entity, With<ConfirmationPopup>>,
    mut tutorial_progress: ResMut<crate::ui::tutorial::resources::TutorialProgress>,
    mut popup_queue: ResMut<crate::ui::notification::NotificationQueue>,
    mut clear_progress_msg: MessageWriter<
        crate::game::achievements::messages::ClearProgressMessage,
    >,
) {
    if !popup_query.is_empty() && back_msgs.read().next().is_some() {
        for entity in &popup_query {
            commands.entity(entity).despawn();
        }
        return;
    }
    for event in button_clicked.read() {
        if let Ok(action) = action_query.get(event.button) {
            if let ConfirmationAction::Confirm(settings_action) = action {
                match settings_action {
                    SettingsButtonAction::ResetTutorials => {
                        tutorial_progress.reset();
                        crate::ui::tutorial::systems::reset_tutorial_progress();
                        popup_queue.push(crate::ui::notification::NotificationEntry::Toast {
                            message: "Tutorials have been reset.",
                        });
                    }
                    SettingsButtonAction::ClearProgress => {
                        crate::config::save_data::clear_progress();
                        clear_progress_msg
                            .write(crate::game::achievements::messages::ClearProgressMessage);
                        popup_queue.push(crate::ui::notification::NotificationEntry::Toast {
                            message: "All progress has been cleared.",
                        });
                    }
                    _ => {}
                }
            }
            // Despawn popup on either Confirm or Cancel
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handles settings button actions when clicked from main menu.
#[allow(clippy::too_many_arguments)]
pub fn settings_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SettingsButtonAction>,
    popup_query: Query<&ConfirmationPopup>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
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
                    channel_change.write(ChannelChangeMessage);
                    next_menu_state.set(MenuState::Landing);
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

/// Handles settings button actions when clicked from pause menu.
pub fn pause_settings_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SettingsButtonAction>,
    popup_query: Query<&ConfirmationPopup>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
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
                    next_pause_menu_state.set(PauseMenuState::Main);
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

/// Handles option button clicks.
pub fn option_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&OptionButtonValue>,
    mut game_config: ResMut<GameConfig>,
) {
    for event in button_clicked.read() {
        if let Ok(value) = button_query.get(event.button) {
            value.apply(&mut game_config);
        }
    }
}

/// Handles slider button clicks for increment/decrement.
pub fn slider_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    down_buttons: Query<&SliderDownButton>,
    up_buttons: Query<&SliderUpButton>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    for event in button_clicked.read() {
        // Check if it's a down button
        if let Ok(button) = down_buttons.get(event.button) {
            let current = button.value.get(&game_config);
            let step = button.value.step();
            let min = button.value.min_value();
            let new_value = (current - step).max(min);
            button.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
        // Check if it's an up button
        else if let Ok(button) = up_buttons.get(event.button) {
            let current = button.value.get(&game_config);
            let step = button.value.step();
            let max = button.value.max_value();
            let new_value = (current + step).min(max);
            button.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}

/// Updates slider text displays when values change.
pub fn update_slider_text(
    game_config: Res<GameConfig>,
    mut slider_texts: Query<(&mut Text, &SliderText)>,
) {
    if game_config.is_changed() {
        for (mut text, slider_text) in &mut slider_texts {
            let value = slider_text.value.get(&game_config);
            text.0 = format!("{}%", (value * 100.0) as u8);
        }
    }
}

/// Updates slider fill widths and handle positions when values change.
pub fn update_sliders(
    game_config: Res<GameConfig>,
    mut slider_fills: Query<(&mut Node, &SliderFill), Without<SliderHandle>>,
    mut slider_handles: Query<(&mut Node, &SliderHandle), Without<SliderFill>>,
) {
    if game_config.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let value = slider_fill.value.get(&game_config);
            let min = slider_fill.value.min_value();
            let max = slider_fill.value.max_value();
            let range = max - min;
            // Normalize to 0-100% range
            let normalized = (value - min) / range;
            node.width = Val::Percent(normalized * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let value = slider_handle.value.get(&game_config);
            let min = slider_handle.value.min_value();
            let max = slider_handle.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * crate::ui::constants::SLIDER_TRACK_WIDTH - 2.0);
        }
    }
}

/// Handles dragging slider handles and clicking on tracks.
///
/// Uses the track's `RelativeCursorPosition` for both click-to-jump and drag
/// tracking. This is immune to scale factor, viewport, and CRT distortion
/// differences between mouse motion units and logical UI pixels.
pub fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut SliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &SliderTrack)>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    // Stop dragging when mouse is released
    if !buttons.pressed(bevy::input::mouse::MouseButton::Left) {
        for (_interaction, mut slider_handle) in &mut slider_handles {
            slider_handle.is_dragging = false;
        }
        return;
    }

    // Check if track was clicked (start dragging)
    if buttons.just_pressed(bevy::input::mouse::MouseButton::Left) {
        for (interaction, _cursor_pos, track) in &slider_tracks {
            if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                // Start dragging the corresponding handle
                for (_handle_interaction, mut slider_handle) in &mut slider_handles {
                    if slider_handle.value == track.value {
                        slider_handle.is_dragging = true;
                    }
                }
            }
        }

        // Also start dragging if the handle itself was clicked
        for (interaction, mut slider_handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                slider_handle.is_dragging = true;
            }
        }
    }

    // While dragging, use the track's RelativeCursorPosition to set the value.
    // This gives pixel-perfect tracking regardless of scale factor or viewport.
    for (_interaction, cursor_pos, track) in &slider_tracks {
        let is_dragging = slider_handles
            .iter()
            .any(|(_, h)| h.value == track.value && h.is_dragging);

        if is_dragging && let Some(pos) = cursor_pos.normalized {
            // RelativeCursorPosition.normalized: center at (0,0),
            // left edge = -0.5, right edge = 0.5
            let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

            let min = track.value.min_value();
            let max = track.value.max_value();
            let range = max - min;
            let new_value = (min + normalized * range).clamp(min, max);

            if (track.value.get(&game_config) - new_value).abs() > f32::EPSILON {
                track.value.set(&mut game_config, new_value);
                slider_adjusted.write(SliderAdjusted);
            }
        }
    }
}

/// Updates selected state styling for option buttons.
pub fn update_selected_options(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut option_buttons: Query<
        (
            Entity,
            &OptionButtonValue,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut ButtonColors,
            Option<&Children>,
        ),
        With<Button>,
    >,
    mut front_query: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<crate::ui::components::ButtonFront>, Without<Button>),
    >,
) {
    if game_config.is_changed() {
        for (entity, value, mut bg, mut border, mut colors, children) in &mut option_buttons {
            let (new_bg, new_border) = if value.is_selected(&game_config) {
                commands
                    .entity(entity)
                    .insert((SelectedOption, crate::ui::components::ButtonActive));
                (SELECTED_BACKGROUND, SELECTED_BORDER)
            } else {
                commands.entity(entity).remove::<SelectedOption>();
                commands
                    .entity(entity)
                    .remove::<crate::ui::components::ButtonActive>();
                (BUTTON_BACKGROUND, BUTTON_BORDER)
            };

            *bg = BackgroundColor(new_bg);
            *border = BorderColor::all(new_border);
            colors.background = new_bg;
            colors.border = new_border;

            // Also update the 3D front face child.
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok((mut front_bg, mut front_border)) = front_query.get_mut(child) {
                        *front_bg = crate::ui::systems::opaque(new_bg).into();
                        *front_border = BorderColor::all(new_border);
                    }
                }
            }
        }
    }
}
