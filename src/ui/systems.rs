//! Shared UI systems used across all menus and screens.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::{ButtonColors, ButtonStyle};
use super::styles::{item_hovered, item_pressed};
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub struct ButtonPressedDown;

/// Run condition that returns true if there are any MouseClicked messages.
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().next().is_some()
}

/// Tracks button press state and sends click events.
///
/// This system handles the core button click detection:
/// - Marks buttons as pressed when interaction becomes Pressed
/// - Sends MouseClicked event when interaction changes from Pressed to non-Pressed (either Hovered or None)
/// - Only sends click event if the button was previously marked as pressed down
///
/// This works for both mouse (Pressed → Hovered → None) and touch (Pressed → None).
pub fn button_click_detection(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, Option<&ButtonPressedDown>),
        (Changed<Interaction>, With<Button>),
    >,
    mut button_clicked: MessageWriter<MouseClicked>,
) {
    for (entity, interaction, pressed_down) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Mark button as pressed down
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered | Interaction::None => {
                // If button was pressed down and is now released, send click event
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                    button_clicked.write(MouseClicked { button: entity });
                }
            }
        }
    }
}

/// Handles button interaction visual feedback for all buttons with `ButtonColors`.
///
/// Updates button background and border colors based on the current
/// interaction state (None, Hovered, or Pressed).
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

// ---------------------------------------------------------------------------
// Page container (shared by settings, progress, instructions)
// ---------------------------------------------------------------------------

/// Dark opaque background for page content containers.
const PAGE_CONTENT_BG: Color = Color::hsla(220.0, 0.08, 0.08, 1.0);

/// Subtle border for page content containers.
const PAGE_CONTENT_BORDER: Color = Color::hsla(0.0, 0.0, 0.18, 1.0);

/// Semi-transparent overlay behind the content container.
const PAGE_OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);

/// Spawns a full-screen page with a semi-transparent overlay and a dark opaque
/// content container inside it. Returns the content container entity so the
/// caller can add children and extra components.
///
/// When `pause_menu` is true, `GlobalZIndex(500)` is added so the page
/// renders above in-game UI.
pub fn spawn_page_container<M: Component>(
    commands: &mut Commands,
    screen_marker: M,
    pause_menu: bool,
    content_overflow: Overflow,
) -> Entity {
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(PAGE_OVERLAY_BG),
        screen_marker,
    ));

    if pause_menu {
        root.insert(GlobalZIndex(500));
    }

    let root_id = root.id();

    let content_id = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(1.0)),
                overflow: content_overflow,
                ..default()
            },
            BackgroundColor(PAGE_CONTENT_BG),
            BorderColor::all(PAGE_CONTENT_BORDER),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .id();

    commands.entity(root_id).add_child(content_id);

    content_id
}

// ---------------------------------------------------------------------------
// Shared Escape key handling
// ---------------------------------------------------------------------------

/// Handles Escape key to return to the main menu landing screen.
pub fn escape_to_landing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(MenuState::Landing);
    }
}

/// Handles Escape key to return to the pause menu main screen.
pub fn escape_to_pause_main(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(PauseMenuState::Main);
    }
}

// ---------------------------------------------------------------------------
// Shared button spawning
// ---------------------------------------------------------------------------

/// Spawns a styled button as a child of the given parent.
///
/// # Arguments
///
/// * `parent` - The parent entity to spawn the button under
/// * `text` - The button label text
/// * `action` - Any component to attach as the button's action identifier
/// * `style` - The `ButtonStyle` configuration for dimensions and colors
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Component,
    style: &ButtonStyle,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(style.width),
                height: Val::Px(style.height),
                border: UiRect::all(Val::Px(style.border_width)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(style.border),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(style.background),
            ButtonColors {
                background: style.background,
                border: style.border,
            },
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont::from_font_size(style.font_size),
                TextColor(style.text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}
