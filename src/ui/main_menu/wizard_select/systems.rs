//! Wizard select screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::config::save_data::{self, load_unified_save};
use crate::config::{ActiveSave, ConfigChanged, GameConfig, WizardType};
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, MenuState};
use crate::ui::components::ButtonColors;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;

/// Sets up the wizard select screen UI.
pub(super) fn setup(mut commands: Commands) {
    // Default preview to the first wizard type
    let default_wizard = WizardType::all()[0];
    commands.insert_resource(SelectedWizardPreview(default_wizard));
    spawn_wizard_type_screen(&mut commands, default_wizard);
}

/// Spawns the wizard type selection UI.
///
/// Layout:
/// ```text
/// ┌────────────────────────────────────────────────────────────┐
/// │ ┌─ Left Panel ──────┐  ┌─ Right: Grid ──────────────────┐ │
/// │ │ Choose Your Path   │  │ ┌────┐ ┌────┐ ┌────┐ ┌────┐  │ │
/// │ │ subtitle           │  │ │card│ │card│ │card│ │card│  │ │
/// │ │                    │  │ └────┘ └────┘ └────┘ └────┘  │ │
/// │ │ ┌──────────────┐   │  │ ┌────┐ ┌────┐ ┌────┐ ┌────┐  │ │
/// │ │ │ Detail Card   │   │  │ │ ?? │ │ ?? │ │ ?? │ │ ?? │  │ │
/// │ │ │ Name          │   │  │ └────┘ └────┘ └────┘ └────┘  │ │
/// │ │ │ Long desc     │   │  │ ...                           │ │
/// │ │ │ Status        │   │  │                               │ │
/// │ │ │ [Play]        │   │  │                               │ │
/// │ │ └──────────────┘   │  └────────────────────────────────┘ │
/// │ │                    │                                     │
/// │ │ [Back]             │                                     │
/// │ └────────────────────┘                                     │
/// └────────────────────────────────────────────────────────────┘
/// ```
fn spawn_wizard_type_screen(commands: &mut Commands, initial_wizard: WizardType) {
    let wizard_types = WizardType::all();
    let initial_save = save_data::get_wizard_by_type(initial_wizard);
    let unlocked_wizard_types = load_unified_save()
        .map(|s| s.player.unlocked_content.wizard_types)
        .unwrap_or_default();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(MARGIN * 1.5)),
                column_gap: Val::Px(MARGIN * 1.5),
                ..default()
            },
            OnWizardSelectScreen,
        ))
        .with_children(|root| {
            // ── Left panel ──────────────────────────────────────
            root.spawn(Node {
                width: Val::Px(LEFT_PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(MARGIN),
                ..default()
            })
            .with_children(|left| {
                // Top group: title + detail card
                left.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(MARGIN),
                    ..default()
                })
                .with_children(|top| {
                    // Title group
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|title_group| {
                        title_group.spawn((
                            Text::new("Choose Your Path"),
                            TextFont {
                                font_size: TITLE_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                        title_group.spawn((
                            Text::new("Select your wizard archetype"),
                            TextFont {
                                font_size: SUBTITLE_FONT_SIZE,
                                ..default()
                            },
                            TextColor(SUBTITLE_COLOR),
                        ));
                    });

                    // Separator
                    top.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            ..default()
                        },
                        BackgroundColor(Color::hsla(40.0, 0.15, 0.15, 1.0)),
                    ));

                    // Detail card
                    spawn_detail_panel(top, initial_wizard, &initial_save);
                });

                // Bottom: back button
                spawn_button(
                    left,
                    "Back",
                    WizardSelectButtonAction::Back,
                    &BACK_BUTTON_STYLE,
                );
            });

            // ── Right side: grid ────────────────────────────────
            root.spawn(Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::FlexEnd,
                align_content: AlignContent::FlexStart,
                column_gap: Val::Px(CARD_GAP),
                row_gap: Val::Px(CARD_GAP),
                max_width: Val::Px(
                    (CARD_WIDTH * GRID_COLUMNS as f32) + (CARD_GAP * (GRID_COLUMNS - 1) as f32),
                ),
                ..default()
            })
            .with_children(|grid| {
                for slot in 0..GRID_SLOTS {
                    if let Some(wizard_type) = wizard_types.get(slot) {
                        let type_name = format!("{:?}", wizard_type);
                        if unlocked_wizard_types.contains(&type_name) {
                            let is_selected = *wizard_type == initial_wizard;
                            spawn_wizard_card(grid, *wizard_type, is_selected);
                        } else {
                            spawn_locked_wizard_card(grid, *wizard_type);
                        }
                    } else {
                        spawn_locked_card(grid);
                    }
                }
            });
        });
}

/// Spawns the detail panel showing expanded info about the selected wizard.
fn spawn_detail_panel(
    parent: &mut ChildSpawnerCommands,
    wizard_type: WizardType,
    existing_save: &Option<save_data::WizardSave>,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(DETAIL_BORDER_WIDTH)),
                ..default()
            },
            BackgroundColor(DETAIL_BG),
            BorderColor::all(DETAIL_BORDER),
            BorderRadius::all(Val::Px(DETAIL_BORDER_RADIUS)),
            DetailPanel,
        ))
        .with_children(|card| {
            // Top: name + long description
            card.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|top| {
                top.spawn((
                    Text::new(wizard_type.display_name()),
                    TextFont {
                        font_size: DETAIL_NAME_FONT_SIZE,
                        ..default()
                    },
                    TextColor(CARD_NAME_COLOR),
                    DetailName,
                ));

                top.spawn((
                    Text::new(wizard_type.long_description()),
                    TextFont {
                        font_size: DETAIL_DESC_FONT_SIZE,
                        ..default()
                    },
                    TextColor(DETAIL_DESC_COLOR),
                    Node {
                        max_width: Val::Px(LEFT_PANEL_WIDTH - 36.0),
                        ..default()
                    },
                    DetailDescription,
                ));
            });

            // Bottom: status + play button
            card.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|bottom| {
                // Status
                let (status_text, status_color) = if let Some(save) = existing_save {
                    (format!("Level {}", save.highest_level_achieved), STAT_COLOR)
                } else {
                    ("New".to_string(), NEW_COLOR)
                };

                bottom.spawn((
                    Text::new(status_text),
                    TextFont {
                        font_size: DETAIL_STATUS_FONT_SIZE,
                        ..default()
                    },
                    TextColor(status_color),
                    DetailStatus,
                ));

                // Play button
                spawn_button(
                    bottom,
                    "Play",
                    WizardSelectButtonAction::Play,
                    &PLAY_BUTTON_STYLE,
                );
            });
        });
}

/// Spawns an unlocked wizard card (compact, for the grid).
fn spawn_wizard_card(
    parent: &mut ChildSpawnerCommands,
    wizard_type: WizardType,
    is_selected: bool,
) {
    let border_color = if is_selected {
        CARD_BORDER_SELECTED
    } else {
        CARD_BORDER
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(CARD_BG),
            BorderColor::all(border_color),
            BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
            ButtonColors {
                background: CARD_BG,
                border: border_color,
            },
            WizardSelectButtonAction::PreviewWizard(wizard_type),
            WizardCard(wizard_type),
        ))
        .with_children(|card| {
            // Wizard name
            card.spawn((
                Text::new(wizard_type.display_name()),
                TextFont {
                    font_size: CARD_NAME_FONT_SIZE,
                    ..default()
                },
                TextColor(CARD_NAME_COLOR),
                TextLayout::new_with_justify(Justify::Center),
            ));

            // Flavor text
            card.spawn((
                Text::new(wizard_type.locked_description()),
                TextFont {
                    font_size: CARD_DESC_FONT_SIZE,
                    ..default()
                },
                TextColor(DESCRIPTION_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Px(CARD_WIDTH - 20.0),
                    ..default()
                },
            ));
        });
}

/// Spawns a locked wizard card showing the wizard name and flavor text, but not interactive.
fn spawn_locked_wizard_card(parent: &mut ChildSpawnerCommands, wizard_type: WizardType) {
    parent
        .spawn((
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(LOCKED_CARD_BG),
            BorderColor::all(LOCKED_CARD_BORDER),
            BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(wizard_type.locked_description()),
                TextFont {
                    font_size: CARD_DESC_FONT_SIZE,
                    ..default()
                },
                TextColor(LOCKED_TEXT_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Px(CARD_WIDTH - 20.0),
                    ..default()
                },
            ));
        });
}

/// Spawns a locked/unavailable card placeholder.
fn spawn_locked_card(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                ..default()
            },
            BackgroundColor(LOCKED_CARD_BG),
            BorderColor::all(LOCKED_CARD_BORDER),
            BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new("???"),
                TextFont {
                    font_size: CARD_NAME_FONT_SIZE,
                    ..default()
                },
                TextColor(LOCKED_TEXT_COLOR),
            ));
        });
}

/// Cleans up the wizard select screen UI when exiting the state.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnWizardSelectScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SelectedWizardPreview>();
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
    mut preview: ResMut<SelectedWizardPreview>,
    mut detail_name: Query<
        &mut Text,
        (
            With<DetailName>,
            Without<DetailDescription>,
            Without<DetailStatus>,
        ),
    >,
    mut detail_desc: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailName>,
            Without<DetailStatus>,
        ),
    >,
    mut detail_status: Query<
        (&mut Text, &mut TextColor),
        (
            With<DetailStatus>,
            Without<DetailName>,
            Without<DetailDescription>,
        ),
    >,
    mut card_borders: Query<(&WizardCard, &mut BorderColor, &mut ButtonColors)>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match *action {
            WizardSelectButtonAction::PreviewWizard(wizard_type) => {
                preview.0 = wizard_type;

                // Update detail panel text
                if let Ok(mut name_text) = detail_name.single_mut() {
                    **name_text = wizard_type.display_name().to_string();
                }
                if let Ok(mut desc_text) = detail_desc.single_mut() {
                    **desc_text = wizard_type.long_description().to_string();
                }
                if let Ok((mut status_text, mut status_color)) = detail_status.single_mut() {
                    let existing_save = save_data::get_wizard_by_type(wizard_type);
                    if let Some(ref save) = existing_save {
                        **status_text = format!("Level {}", save.highest_level_achieved);
                        status_color.0 = STAT_COLOR;
                    } else {
                        **status_text = "New".to_string();
                        status_color.0 = NEW_COLOR;
                    }
                }

                // Update card border highlights and button colors
                for (card, mut border, mut colors) in card_borders.iter_mut() {
                    let new_border = if card.0 == wizard_type {
                        CARD_BORDER_SELECTED
                    } else {
                        CARD_BORDER
                    };
                    *border = BorderColor::all(new_border);
                    colors.border = new_border;
                }
            }
            WizardSelectButtonAction::Play => {
                let wizard_type = preview.0;
                if save_data::load_wizard_type_into_config(
                    wizard_type,
                    &mut game_config,
                    &mut active_save,
                ) {
                    config_events.write(ConfigChanged);
                    next_app_state.set(AppState::Loading);
                } else {
                    let wizard_id = save_data::create_wizard(wizard_type);
                    game_config.wizard_type = wizard_type;
                    game_config.current_level = 1;
                    game_config.highest_level_achieved = 1;
                    game_config.efficiency_ratios = Default::default();
                    game_config.action_bar_slots = [None; 5];
                    active_save.0 = Some(wizard_id);
                    config_events.write(ConfigChanged);
                    next_app_state.set(AppState::Loading);
                }
            }
            WizardSelectButtonAction::Back => {
                next_menu_state.set(MenuState::Landing);
            }
        }
    }
}

/// Handles keyboard input in the wizard select screen.
pub(super) fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_menu_state.set(MenuState::Landing);
    }
}
