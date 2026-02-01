//! Landing screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::state::{AppState, MenuState};
use crate::ui::systems::spawn_button;

use super::components::{MenuButtonAction, OnLandingScreen};
use super::constants::{BUTTON_STYLE, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE};

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub(super) struct ButtonPressedDown;

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
                "Start Game",
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
/// Uses a marker component to ensure buttons only trigger on release after being pressed.
pub fn button_action(
    mut commands: Commands,
    interaction_query: Query<
        (
            Entity,
            &Interaction,
            &MenuButtonAction,
            Option<&ButtonPressedDown>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    for (entity, interaction, action, pressed_down) in &interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Mark button as pressed down
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                // Only trigger action if button was previously pressed
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        MenuButtonAction::StartGame => {
                            next_app_state.set(AppState::InGame);
                        }
                        MenuButtonAction::Settings => {
                            next_menu_state.set(MenuState::Settings);
                        }
                        MenuButtonAction::Changelog => {
                            next_menu_state.set(MenuState::Changelog);
                        }
                    }
                }
            }
            Interaction::None => {
                // Trigger action on release (touch goes Pressed → None, skipping Hovered)
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        MenuButtonAction::StartGame => {
                            next_app_state.set(AppState::InGame);
                        }
                        MenuButtonAction::Settings => {
                            next_menu_state.set(MenuState::Settings);
                        }
                        MenuButtonAction::Changelog => {
                            next_menu_state.set(MenuState::Changelog);
                        }
                    }
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
