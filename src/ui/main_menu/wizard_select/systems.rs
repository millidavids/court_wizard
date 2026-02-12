//! Wizard select screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::config::save_data;
use crate::config::{ActiveSave, ConfigChanged, GameConfig, WizardType};
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, MenuState};
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;

/// Sets up the wizard select screen UI.
pub(super) fn setup(mut commands: Commands) {
    spawn_wizard_type_screen(&mut commands);
}

/// Spawns the wizard type selection UI.
/// Shows all wizard types with their stats if a save exists, or "New" if not.
fn spawn_wizard_type_screen(commands: &mut Commands) {
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
                    // font removed (using default),
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN * 2.0)),
                    ..default()
                },
            ));

            for wizard_type in WizardType::all() {
                let existing_save = save_data::get_wizard_by_type(*wizard_type);

                spawn_button(
                    parent,
                    wizard_type.display_name(),
                    WizardSelectButtonAction::SelectWizard(*wizard_type),
                    &BUTTON_STYLE,
                );

                // Description text
                parent.spawn((
                    Text::new(wizard_type.description()),
                    TextFont {
                        // font removed (using default),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(DESCRIPTION_COLOR),
                ));

                // Stats or "New" indicator
                if let Some(ref save) = existing_save {
                    parent.spawn((
                        Text::new(format!("Highest Level: {}", save.highest_level_achieved)),
                        TextFont {
                            // font removed (using default),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(STAT_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(MARGIN * 0.5)),
                            ..default()
                        },
                    ));
                } else {
                    parent.spawn((
                        Text::new("New"),
                        TextFont {
                            // font removed (using default),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(NEW_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(MARGIN * 0.5)),
                            ..default()
                        },
                    ));
                }
            }

            spawn_button(
                parent,
                "Back",
                WizardSelectButtonAction::Back,
                &BUTTON_STYLE,
            );
        });
}

/// Cleans up the wizard select screen UI when exiting the state.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnWizardSelectScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
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
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                WizardSelectButtonAction::SelectWizard(wizard_type) => {
                    // If a save exists for this type, load it directly
                    if save_data::load_wizard_type_into_config(
                        *wizard_type,
                        &mut game_config,
                        &mut active_save,
                    ) {
                        config_events.write(ConfigChanged);
                        next_app_state.set(AppState::Loading);
                    } else {
                        // No save exists — create one and load it
                        let wizard_id = save_data::create_wizard(*wizard_type);
                        game_config.wizard_type = *wizard_type;
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
