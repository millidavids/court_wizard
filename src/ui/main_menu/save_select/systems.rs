//! Save select screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::config::save_data;
use crate::config::{ActiveSave, ConfigChanged, GameConfig};
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, MenuState};
use crate::ui::resources::CustomFont;
use crate::ui::systems::spawn_button;

use super::components::{OnSaveSelectScreen, SaveSelectButtonAction};
use super::constants::{BUTTON_STYLE, DELETE_BUTTON_STYLE, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE};

/// Sets up the save select screen UI.
pub(super) fn setup(mut commands: Commands, custom_font: Res<CustomFont>) {
    let summaries = save_data::load_all_summaries();

    // Root container - full screen, centered content in a column
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            OnSaveSelectScreen,
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("Choose Save File"),
                TextFont {
                    font: custom_font.handle.clone(),
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN * 2.0)),
                    ..default()
                },
            ));

            // Save slot buttons
            for summary in &summaries {
                let button_text = format!(
                    "{} ({} Lv{})",
                    summary.wizard_name,
                    summary.wizard_type.display_name(),
                    summary.current_level,
                );

                // Row container for load + delete buttons
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_button(
                            row,
                            &button_text,
                            SaveSelectButtonAction::LoadSave(summary.slot),
                            &BUTTON_STYLE,
                            &custom_font,
                        );

                        spawn_button(
                            row,
                            "Delete",
                            SaveSelectButtonAction::DeleteSave(summary.slot),
                            &DELETE_BUTTON_STYLE,
                            &custom_font,
                        );
                    });
            }

            // Back button
            spawn_button(
                parent,
                "Back",
                SaveSelectButtonAction::Back,
                &BUTTON_STYLE,
                &custom_font,
            );
        });
}

/// Cleans up the save select screen UI when exiting the state.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnSaveSelectScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles save select button actions.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SaveSelectButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut game_config: ResMut<GameConfig>,
    mut active_save: ResMut<ActiveSave>,
    mut config_events: MessageWriter<ConfigChanged>,
    mut commands: Commands,
    screen_query: Query<Entity, With<OnSaveSelectScreen>>,
    custom_font: Res<CustomFont>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                SaveSelectButtonAction::LoadSave(slot) => {
                    if save_data::load_save_into_config(*slot, &mut game_config, &mut active_save) {
                        config_events.write(ConfigChanged);
                        next_app_state.set(AppState::Loading);
                    }
                }
                SaveSelectButtonAction::DeleteSave(slot) => {
                    save_data::delete_slot(*slot);

                    // Check if any saves remain
                    let remaining = save_data::load_all_summaries();
                    if remaining.is_empty() {
                        // No saves left, go back to landing
                        next_menu_state.set(MenuState::Landing);
                    } else {
                        // Rebuild the UI by despawning and re-running setup
                        for entity in &screen_query {
                            commands.entity(entity).despawn();
                        }
                        rebuild_save_select_ui(&mut commands, &custom_font, &remaining);
                    }
                }
                SaveSelectButtonAction::Back => {
                    next_menu_state.set(MenuState::Landing);
                }
            }
        }
    }
}

/// Rebuilds the save select UI after a delete operation.
fn rebuild_save_select_ui(
    commands: &mut Commands,
    custom_font: &CustomFont,
    summaries: &[save_data::SaveSummary],
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            OnSaveSelectScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Choose Save File"),
                TextFont {
                    font: custom_font.handle.clone(),
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN * 2.0)),
                    ..default()
                },
            ));

            for summary in summaries {
                let button_text = format!(
                    "{} ({} Lv{})",
                    summary.wizard_name,
                    summary.wizard_type.display_name(),
                    summary.current_level,
                );

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_button(
                            row,
                            &button_text,
                            SaveSelectButtonAction::LoadSave(summary.slot),
                            &BUTTON_STYLE,
                            &custom_font,
                        );

                        spawn_button(
                            row,
                            "Delete",
                            SaveSelectButtonAction::DeleteSave(summary.slot),
                            &DELETE_BUTTON_STYLE,
                            &custom_font,
                        );
                    });
            }

            spawn_button(
                parent,
                "Back",
                SaveSelectButtonAction::Back,
                &BUTTON_STYLE,
                &custom_font,
            );
        });
}

/// Handles keyboard input in the save select screen.
pub(super) fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_menu_state.set(MenuState::Landing);
    }
}
