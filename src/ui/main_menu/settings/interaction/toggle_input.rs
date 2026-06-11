use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};

use super::super::builders::spawn_confirmation_popup;
use super::super::components::{
    ButtonColors, ConfirmationAction, ConfirmationPopup, OptionButtonValue, SelectedOption,
    SettingsButtonAction,
};
use super::super::constants::{
    BUTTON_BACKGROUND, BUTTON_BORDER, SELECTED_BACKGROUND, SELECTED_BORDER,
};

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
