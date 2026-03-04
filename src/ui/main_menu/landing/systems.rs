//! Landing screen systems.

use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::MenuState;
use crate::ui::systems::spawn_button;

use super::components::{MenuButtonAction, OnLandingScreen};
use super::constants::{BUTTON_STYLE, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE};

/// Sets up the landing screen UI.
///
/// Spawns the root UI node containing the title and menu buttons.
/// All spawned entities are marked with `OnLandingScreen` for cleanup.
pub fn setup(mut commands: Commands) {
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
            OnLandingScreen,
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("Court Wizard"),
                TextFont::from_font_size(TITLE_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN * 2.0)),
                    ..default()
                },
            ));

            // Play button - goes to wizard select screen
            spawn_button(parent, "Play", MenuButtonAction::Play, &BUTTON_STYLE);

            // Settings button
            spawn_button(
                parent,
                "Settings",
                MenuButtonAction::Settings,
                &BUTTON_STYLE,
            );

            // Progress button
            spawn_button(
                parent,
                "Progress",
                MenuButtonAction::Progress,
                &BUTTON_STYLE,
            );

            // Changelog button
            spawn_button(
                parent,
                "Changelog",
                MenuButtonAction::Changelog,
                &BUTTON_STYLE,
            );

            // Instructions button
            spawn_button(
                parent,
                "Instructions",
                MenuButtonAction::Instructions,
                &BUTTON_STYLE,
            );

            // Credits button
            spawn_button(
                parent,
                "Credits",
                MenuButtonAction::Credits,
                &BUTTON_STYLE,
            );

            // Multiplayer button
            spawn_button(
                parent,
                "Multiplayer",
                MenuButtonAction::Multiplayer,
                &BUTTON_STYLE,
            );
        });
}

/// Handles menu button actions.
///
/// Triggers state transitions based on the button's `MenuButtonAction` component.
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MenuButtonAction>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            match action {
                MenuButtonAction::Play => {
                    next_menu_state.set(MenuState::WizardSelect);
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
                MenuButtonAction::Progress => {
                    next_menu_state.set(MenuState::Progress);
                }
                MenuButtonAction::Credits => {
                    next_menu_state.set(MenuState::Credits);
                }
                MenuButtonAction::Multiplayer => {
                    next_menu_state.set(MenuState::Multiplayer);
                }
            }
        }
    }
}

