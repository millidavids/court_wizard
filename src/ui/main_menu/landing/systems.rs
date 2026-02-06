//! Landing screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::game::input::events::MouseClicked;
use crate::state::{AppState, MenuState};
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
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN * 2.0)),
                    ..default()
                },
            ));

            // Start Game button
            spawn_button(
                parent,
                "Begin, Wizard",
                MenuButtonAction::StartGame,
                &BUTTON_STYLE,
            );

            // Settings button
            spawn_button(
                parent,
                "Settings",
                MenuButtonAction::Settings,
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
        });
}

/// Cleans up the landing screen UI when exiting the state.
///
/// Despawns all entities marked with `OnLandingScreen`.
pub fn cleanup(mut commands: Commands, landing_items: Query<Entity, With<OnLandingScreen>>) {
    for entity in &landing_items {
        commands.entity(entity).despawn();
    }
}

/// Handles menu button actions.
///
/// Triggers state transitions based on the button's `MenuButtonAction` component.
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MenuButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MenuButtonAction::StartGame => {
                    next_app_state.set(AppState::Loading);
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
            }
        }
    }
}

/// Handles keyboard input in the landing screen.
pub fn keyboard_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        // Do nothing - already at top level
    }
}
