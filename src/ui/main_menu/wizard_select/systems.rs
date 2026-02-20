//! Wizard select screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::config::save_data::{self, load_unified_save};
use crate::config::{ActiveSave, ConfigChanged, GameConfig, WizardType};
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, MenuState};
use crate::ui::components::ButtonColors;
use crate::ui::systems::spawn_button;

use super::super::wizard_select_shared::{self as shared, grid_container_node};
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
                    shared::spawn_title_group(
                        top,
                        "Choose Your Path",
                        "Select your wizard archetype",
                    );

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
            root.spawn(grid_container_node())
                .with_children(|grid| {
                    for slot in 0..GRID_SLOTS {
                        if let Some(wizard_type) = wizard_types.get(slot) {
                            let type_name = format!("{:?}", wizard_type);
                            if unlocked_wizard_types.contains(&type_name) {
                                let is_selected = *wizard_type == initial_wizard;
                                shared::spawn_wizard_card(
                                    grid,
                                    *wizard_type,
                                    is_selected,
                                    WizardSelectButtonAction::PreviewWizard(*wizard_type),
                                );
                            } else {
                                shared::spawn_locked_wizard_card(grid, *wizard_type);
                            }
                        } else {
                            shared::spawn_locked_card(grid);
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
    let (status_text, status_color) = if let Some(save) = existing_save {
        (format!("Level {}", save.highest_level_achieved), STAT_COLOR)
    } else {
        ("New".to_string(), NEW_COLOR)
    };

    shared::spawn_detail_panel_container(parent, |card| {
        shared::spawn_detail_panel_top(card, wizard_type);

        // Bottom: status + play button
        card.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|bottom| {
            bottom.spawn((
                Text::new(status_text),
                TextFont {
                    font_size: DETAIL_STATUS_FONT_SIZE,
                    ..default()
                },
                TextColor(status_color),
                DetailStatus,
            ));

            spawn_button(
                bottom,
                "Play",
                WizardSelectButtonAction::Play,
                &PLAY_BUTTON_STYLE,
            );
        });
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
#[allow(clippy::too_many_arguments)]
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

                shared::update_detail_panel_text(
                    wizard_type,
                    &mut detail_name,
                    &mut detail_desc,
                );

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

                shared::update_card_borders(wizard_type, &mut card_borders);
            }
            WizardSelectButtonAction::Play => {
                let wizard_type = preview.0;
                // Go directly to MetaGame (wizard tower) — no Loading needed
                if save_data::load_wizard_type_into_config(
                    wizard_type,
                    &mut game_config,
                    &mut active_save,
                ) {
                    config_events.write(ConfigChanged);
                    next_app_state.set(AppState::MetaGame);
                } else {
                    let wizard_id = save_data::create_wizard(wizard_type);
                    game_config.wizard_type = wizard_type;
                    game_config.current_level = 1;
                    game_config.highest_level_achieved = 1;
                    game_config.efficiency_ratios = Default::default();
                    game_config.action_bar_slots = [None; 5];
                    active_save.0 = Some(wizard_id);
                    config_events.write(ConfigChanged);
                    next_app_state.set(AppState::MetaGame);
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
