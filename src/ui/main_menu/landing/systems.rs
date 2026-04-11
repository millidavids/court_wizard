//! Landing screen systems.

use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, MenuState};
use crate::ui::systems::{spawn_button, spawn_title_with_shadow};

use super::components::{MenuButtonAction, OnLandingScreen};
use super::constants::{BUTTON_STYLE, BUTTONS_LEFT_PADDING, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE};

/// Sets up the landing screen UI.
pub fn setup(mut commands: Commands) {
    // Root container - horizontal layout, buttons left, title right
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::left(Val::Px(BUTTONS_LEFT_PADDING)),
                ..default()
            },
            OnLandingScreen,
        ))
        .with_children(|parent| {
            // Left side: button column
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(MARGIN),
                    ..default()
                })
                .with_children(|buttons| {
                    spawn_button(buttons, "Wizard Tower", MenuButtonAction::WizardTower, &BUTTON_STYLE);
                    spawn_button(
                        buttons,
                        "Settings",
                        MenuButtonAction::Settings,
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Compendium",
                        MenuButtonAction::Compendium,
                        &BUTTON_STYLE,
                    );
                    spawn_button(buttons, "Manual", MenuButtonAction::Manual, &BUTTON_STYLE);
                    spawn_button(buttons, "Exit", MenuButtonAction::Exit, &BUTTON_STYLE);
                });

            // Right side: title centered in remaining space
            parent
                .spawn(Node {
                    flex_grow: 1.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|title_area| {
                    spawn_title_with_shadow(
                        title_area,
                        "Court\nWizard",
                        TITLE_FONT_SIZE,
                        TEXT_COLOR,
                        Node::default(),
                    );
                });
        });
}

/// Handles menu button actions.
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MenuButtonAction>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut exit: MessageWriter<AppExit>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            match action {
                MenuButtonAction::WizardTower => {
                    next_app_state.set(AppState::MetaGame);
                }
                MenuButtonAction::Settings => {
                    next_menu_state.set(MenuState::Settings);
                }
                MenuButtonAction::Manual => {
                    next_menu_state.set(MenuState::Manual);
                }
                MenuButtonAction::Compendium => {
                    next_menu_state.set(MenuState::Compendium);
                }
                MenuButtonAction::Exit => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}
