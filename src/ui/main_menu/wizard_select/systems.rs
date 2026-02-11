//! Wizard select screen systems.

use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::config::save_data;
use crate::config::{ActiveSave, ConfigChanged, GameConfig, WizardType};
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, MenuState};
use crate::ui::resources::CustomFont;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;

/// Sets up the wizard select screen UI and state resource.
pub(super) fn setup(mut commands: Commands, custom_font: Res<CustomFont>) {
    commands.insert_resource(WizardSelectState::default());

    let has_available_slot = save_data::find_next_available_slot().is_some();

    spawn_wizard_type_screen(&mut commands, &custom_font, has_available_slot);
}

/// Spawns the wizard type selection UI (phase 1).
fn spawn_wizard_type_screen(
    commands: &mut Commands,
    custom_font: &CustomFont,
    has_available_slot: bool,
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
            OnWizardSelectScreen,
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("Choose Your Path"),
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

            if has_available_slot {
                for wizard_type in WizardType::all() {
                    spawn_button(
                        parent,
                        wizard_type.display_name(),
                        WizardSelectButtonAction::SelectWizard(*wizard_type),
                        &BUTTON_STYLE,
                        custom_font,
                    );
                    // Description text below each archetype button
                    parent.spawn((
                        Text::new(wizard_type.description()),
                        TextFont {
                            font: custom_font.handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(DESCRIPTION_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(MARGIN)),
                            ..default()
                        },
                    ));
                }
            } else {
                parent.spawn((
                    Text::new("All save slots are full."),
                    TextFont {
                        font: custom_font.handle.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(WARNING_COLOR),
                    Node {
                        margin: UiRect::bottom(Val::Px(MARGIN)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new("Delete a save to start a new game."),
                    TextFont {
                        font: custom_font.handle.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::bottom(Val::Px(MARGIN)),
                        ..default()
                    },
                ));
            }

            spawn_button(
                parent,
                "Back",
                WizardSelectButtonAction::Back,
                &BUTTON_STYLE,
                custom_font,
            );
        });
}

/// Spawns the name input UI (phase 2).
fn spawn_name_input_screen(
    commands: &mut Commands,
    custom_font: &CustomFont,
    wizard_type: WizardType,
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
            OnWizardSelectScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(format!("Name Your {}", wizard_type.display_name())),
                TextFont {
                    font: custom_font.handle.clone(),
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN)),
                    ..default()
                },
            ));

            // Name input field (visual box with text inside)
            parent
                .spawn((
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(55.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::horizontal(Val::Px(15.0)),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(INPUT_BACKGROUND),
                    BorderColor::all(INPUT_BORDER),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|field| {
                    // The editable text (updated by keyboard_input system)
                    field.spawn((
                        Text::new(""),
                        TextFont {
                            font: custom_font.handle.clone(),
                            font_size: INPUT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        NameInputDisplay,
                    ));
                });

            // Error message (hidden by default)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font: custom_font.handle.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(ERROR_COLOR),
                ErrorDisplay,
            ));

            // Confirm button
            spawn_button(
                parent,
                "Confirm",
                WizardSelectButtonAction::ConfirmName,
                &BUTTON_STYLE,
                custom_font,
            );

            // Back button (returns to wizard type selection)
            spawn_button(
                parent,
                "Back",
                WizardSelectButtonAction::Back,
                &BUTTON_STYLE,
                custom_font,
            );
        });
}

/// Cleans up the wizard select screen UI when exiting the state.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnWizardSelectScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<WizardSelectState>();
}

/// Handles wizard select button actions.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardSelectButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut game_config: ResMut<GameConfig>,
    mut active_save: ResMut<ActiveSave>,
    mut config_events: MessageWriter<ConfigChanged>,
    mut state: ResMut<WizardSelectState>,
    mut commands: Commands,
    screen_query: Query<Entity, With<OnWizardSelectScreen>>,
    custom_font: Res<CustomFont>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                WizardSelectButtonAction::SelectWizard(wizard_type) => {
                    // Move to name input phase
                    state.selected_type = Some(*wizard_type);
                    state.name_input.clear();
                    state.error_message = None;

                    // Rebuild UI for name input
                    for entity in &screen_query {
                        commands.entity(entity).despawn();
                    }
                    spawn_name_input_screen(&mut commands, &custom_font, *wizard_type);
                }
                WizardSelectButtonAction::ConfirmName => {
                    if let Some(wizard_type) = state.selected_type {
                        if let Some(error) = validate_and_create_save(
                            &state,
                            wizard_type,
                            &mut game_config,
                            &mut active_save,
                        ) {
                            state.error_message = Some(error);
                        } else {
                            config_events.write(ConfigChanged);
                            next_app_state.set(AppState::Loading);
                        }
                    }
                }
                WizardSelectButtonAction::Back => {
                    if state.selected_type.is_some() {
                        // Go back to wizard type selection
                        state.selected_type = None;
                        state.name_input.clear();
                        state.error_message = None;

                        for entity in &screen_query {
                            commands.entity(entity).despawn();
                        }
                        let has_slot = save_data::find_next_available_slot().is_some();
                        spawn_wizard_type_screen(&mut commands, &custom_font, has_slot);
                    } else {
                        // Go back to landing
                        next_menu_state.set(MenuState::Landing);
                    }
                }
            }
        }
    }
}

/// Validates the name and creates the save. Returns an error string if validation fails.
fn validate_and_create_save(
    state: &WizardSelectState,
    wizard_type: WizardType,
    config: &mut GameConfig,
    active_save: &mut ActiveSave,
) -> Option<String> {
    let name = state.name_input.trim().to_string();

    if name.is_empty() {
        return Some("Please enter a name.".to_string());
    }

    if save_data::is_name_taken(&name) {
        return Some("That name is already taken.".to_string());
    }

    let Some(slot) = save_data::find_next_available_slot() else {
        return Some("No save slots available.".to_string());
    };

    // Set up the new save
    config.wizard_name = name;
    config.wizard_type = wizard_type;
    config.current_level = 1;
    config.highest_level_achieved = 1;
    config.efficiency_ratios = HashMap::new();
    config.action_bar_slots = [None; 5];
    active_save.0 = Some(slot);

    save_data::save_config_to_active_slot(config, active_save);

    None
}

/// Handles keyboard input for name entry and navigation.
pub(super) fn keyboard_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    state: Option<ResMut<WizardSelectState>>,
    mut name_display: Query<&mut Text, (With<NameInputDisplay>, Without<ErrorDisplay>)>,
    mut error_display: Query<&mut Text, (With<ErrorDisplay>, Without<NameInputDisplay>)>,
) {
    let Some(mut state) = state else { return };

    if state.selected_type.is_some() {
        // Name input phase: handle character input
        for event in keyboard_events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            match &event.logical_key {
                Key::Backspace => {
                    state.name_input.pop();
                    state.error_message = None;
                }
                Key::Character(c) => {
                    if state.name_input.len() < MAX_NAME_LENGTH
                        && c.chars()
                            .all(|ch| ch.is_alphanumeric() || ch == ' ' || ch == '-' || ch == '_')
                    {
                        state.name_input.push_str(c);
                        state.error_message = None;
                    }
                }
                Key::Escape => {
                    // Back action handled via button_action, but also support Escape here
                    // to go back to wizard type selection
                    state.selected_type = None;
                    state.name_input.clear();
                    state.error_message = None;
                    // Note: UI rebuild happens in button_action, so we just clear state.
                    // The Escape key for going back to type selection is handled in button_action
                    // via the Back button. For a cleaner UX, we let button_action handle the rebuild.
                    return;
                }
                _ => {}
            }
        }

        // Update the displayed name text
        if let Ok(mut text) = name_display.single_mut() {
            let display = if state.name_input.is_empty() {
                "_".to_string()
            } else {
                format!("{}_", state.name_input)
            };
            **text = display;
        }

        // Update error display
        if let Ok(mut text) = error_display.single_mut() {
            **text = state.error_message.clone().unwrap_or_default();
        }
    } else {
        // Wizard type selection phase: Escape goes to landing
        if keyboard.just_pressed(KeyCode::Escape) {
            next_menu_state.set(MenuState::Landing);
        }
    }
}
