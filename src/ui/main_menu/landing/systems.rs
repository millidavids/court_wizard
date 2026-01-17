//! Landing screen systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::state::{AppState, MenuState};

use crate::ui::styles::{item_hovered, item_pressed};

use super::components::{ButtonColors, MenuButtonAction, OnLandingScreen};
use super::styles::{
    BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH, BUTTON_FONT_SIZE, BUTTON_HEIGHT,
    BUTTON_WIDTH, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE,
};

/// Sets up the landing screen UI.
///
/// Spawns the root UI node containing the title and menu buttons.
/// All spawned entities are marked with `OnLandingScreen` for cleanup.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for spawning entities
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
                Text::new("The Game"),
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
            spawn_button(parent, "Start Game", MenuButtonAction::StartGame);

            // Settings button
            spawn_button(parent, "Settings", MenuButtonAction::Settings);
        });
}

/// Spawns a menu button with the given text and action.
///
/// # Arguments
///
/// * `parent` - The parent entity spawner to spawn the button under
/// * `text` - The button label text
/// * `action` - The action to trigger when the button is pressed
fn spawn_button(parent: &mut ChildSpawnerCommands, text: &str, action: MenuButtonAction) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(BUTTON_WIDTH),
                height: Val::Px(BUTTON_HEIGHT),
                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(BUTTON_BORDER),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(BUTTON_BACKGROUND),
            ButtonColors {
                background: BUTTON_BACKGROUND,
                border: BUTTON_BORDER,
            },
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font_size: BUTTON_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

/// Cleans up the landing screen UI when exiting the state.
///
/// Despawns all entities marked with `OnLandingScreen`.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for despawning entities
/// * `landing_items` - Query for all entities with the `OnLandingScreen` marker
pub fn cleanup(mut commands: Commands, landing_items: Query<Entity, With<OnLandingScreen>>) {
    for entity in &landing_items {
        commands.entity(entity).despawn();
    }
}

/// Handles button interaction visual feedback.
///
/// Updates button background and border colors based on the current
/// interaction state (None, Hovered, or Pressed).
///
/// # Arguments
///
/// * `interaction_query` - Query for buttons with changed interaction state
#[allow(clippy::type_complexity)] // Complex query types are common in Bevy UI systems
pub fn button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ButtonColors,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, colors, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = item_pressed(colors.background).into();
                *border_color = BorderColor::all(item_pressed(colors.border));
            }
            Interaction::Hovered => {
                *bg_color = item_hovered(colors.background).into();
                *border_color = BorderColor::all(item_hovered(colors.border));
            }
            Interaction::None => {
                *bg_color = colors.background.into();
                *border_color = BorderColor::all(colors.border);
            }
        }
    }
}

/// Handles menu button actions.
///
/// Triggers state transitions based on the button's `MenuButtonAction` component.
///
/// # Arguments
///
/// * `interaction_query` - Query for buttons with changed interaction and an action
/// * `next_app_state` - Resource for transitioning the `AppState`
/// * `next_menu_state` - Resource for transitioning the `MenuState`
#[allow(clippy::type_complexity)] // Complex query types are common in Bevy UI systems
pub fn button_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::StartGame => {
                    next_app_state.set(AppState::InGame);
                }
                MenuButtonAction::Settings => {
                    next_menu_state.set(MenuState::Settings);
                }
            }
        }
    }
}

/// Handles keyboard input in the landing screen.
///
/// - Escape: Does nothing (already at top level)
///
/// # Arguments
///
/// * `keyboard` - Keyboard input resource
pub fn keyboard_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        // Do nothing - already at top level
    }
}
