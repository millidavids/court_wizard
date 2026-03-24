//! Landing screen systems.

use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::MenuState;
use crate::ui::systems::spawn_button;

use super::components::{MenuButtonAction, OnLandingScreen};
use super::constants::{
    BUTTONS_LEFT_PADDING, BUTTON_STYLE, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE, TITLE_SHADOW_COLOR,
    TITLE_SHADOW_OFFSET,
};

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
                    spawn_button(buttons, "Play", MenuButtonAction::Play, &BUTTON_STYLE);
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
                    spawn_button(
                        buttons,
                        "Changelog",
                        MenuButtonAction::Changelog,
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Instructions",
                        MenuButtonAction::Instructions,
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Credits",
                        MenuButtonAction::Credits,
                        &BUTTON_STYLE,
                    );
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
                    // Wrapper for title + shadow (relative positioning anchor)
                    title_area
                        .spawn(Node {
                            position_type: PositionType::Relative,
                            ..default()
                        })
                        .with_children(|wrapper| {
                            wrapper.spawn((
                                Text::new("Court\nWizard"),
                                TextFont::from_font_size(TITLE_FONT_SIZE),
                                TextColor(TITLE_SHADOW_COLOR),
                                TextLayout::new_with_justify(Justify::Center),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(TITLE_SHADOW_OFFSET),
                                    top: Val::Px(TITLE_SHADOW_OFFSET),
                                    ..default()
                                },
                            ));

                            // Main title text
                            wrapper.spawn((
                                Text::new("Court\nWizard"),
                                TextFont::from_font_size(TITLE_FONT_SIZE),
                                TextColor(TEXT_COLOR),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });
                });
        });
}

/// Handles menu button actions.
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MenuButtonAction>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut exit: MessageWriter<AppExit>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            match action {
                MenuButtonAction::Play => {
                    next_menu_state.set(MenuState::GameModeSelect);
                }
                MenuButtonAction::Settings => {
                    next_menu_state.set(MenuState::Settings);
                }
                MenuButtonAction::Changelog => {
                    next_menu_state.set(MenuState::Changelog);
                }
                MenuButtonAction::Instructions => {
                    next_menu_state.set(MenuState::Instructions);
                }
                MenuButtonAction::Compendium => {
                    next_menu_state.set(MenuState::Compendium);
                }
                MenuButtonAction::Credits => {
                    next_menu_state.set(MenuState::Credits);
                }
                MenuButtonAction::Exit => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}
