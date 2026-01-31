//! Shared UI systems used across all menus and screens.

use bevy::prelude::*;

use super::components::{ButtonColors, ButtonStyle};
use super::styles::{item_hovered, item_pressed};

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
                TextFont {
                    font_size: style.font_size,
                    ..default()
                },
                TextColor(style.text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}
