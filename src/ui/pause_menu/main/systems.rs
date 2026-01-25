//! Pause menu main screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::state::{AppState, InGameState, PauseMenuState};
use crate::ui::systems::spawn_button;

use super::components::{OnPauseMainScreen, PauseMenuButtonAction};
use super::constants::{BUTTON_STYLE, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE};

/// Sets up the pause menu main screen UI.
///
/// Spawns the root UI node containing the title and menu buttons.
/// All spawned entities are marked with `OnPauseMainScreen` for cleanup.
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
            OnPauseMainScreen,
            // Semi-transparent dark background to dim the game behind
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            GlobalZIndex(500), // Above game, below brightness overlay
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("Paused"),
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

            // Continue button
            spawn_button(
                parent,
                "Continue",
                PauseMenuButtonAction::Continue,
                &BUTTON_STYLE,
            );

            // Settings button
            spawn_button(
                parent,
                "Settings",
                PauseMenuButtonAction::Settings,
                &BUTTON_STYLE,
            );

            // Exit button
            spawn_button(
                parent,
                "Exit to Menu",
                PauseMenuButtonAction::Exit,
                &BUTTON_STYLE,
            );
        });
}

/// Cleans up the pause menu main screen UI when exiting the state.
///
/// Despawns all entities marked with `OnPauseMainScreen`.
pub fn cleanup(mut commands: Commands, main_items: Query<Entity, With<OnPauseMainScreen>>) {
    for entity in &main_items {
        commands.entity(entity).despawn();
    }
}

/// Handles pause menu button actions.
///
/// Triggers state transitions based on the button's `PauseMenuButtonAction` component.
pub fn button_action(
    interaction_query: Query<
        (&Interaction, &PauseMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                PauseMenuButtonAction::Continue => {
                    next_in_game_state.set(InGameState::Running);
                }
                PauseMenuButtonAction::Settings => {
                    next_pause_menu_state.set(PauseMenuState::Settings);
                }
                PauseMenuButtonAction::Exit => {
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

/// Handles keyboard input in the pause menu.
///
/// - Escape: Resume game (same as Continue button)
pub fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Running);
    }
}
