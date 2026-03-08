//! Systems for credits screen.

use bevy::prelude::*;

use super::components::{OnCreditsScreen, ScrollableCreditsContainer, SpriteCreditsButton};
use crate::game::input::messages::MouseClicked;
use crate::ui::components::{BackButton, ButtonStyle};
use crate::ui::main_menu::landing::constants::{
    BACK_BUTTON_STYLE, BUTTON_BACKGROUND, BUTTON_BORDER, TEXT_COLOR,
};
use crate::ui::systems::{spawn_button, spawn_page_container};

const CREDITS_TEXT: &str = include_str!("../../../../CREDITS.md");

const SPRITE_CREDITS_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 280.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 24.0,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};

/// Strips markdown link syntax `[text](url)` → `text` from the given string.
fn strip_markdown_links(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            // Look for closing ] followed by (
            if let Some(bracket_end) = chars[i + 1..].iter().position(|&c| c == ']') {
                let bracket_end = i + 1 + bracket_end;
                if bracket_end + 1 < chars.len()
                    && chars[bracket_end + 1] == '('
                    && let Some(paren_end) = chars[bracket_end + 2..].iter().position(|&c| c == ')')
                {
                    let paren_end = bracket_end + 2 + paren_end;
                    // Extract link text only
                    let link_text: String = chars[i + 1..bracket_end].iter().collect();
                    result.push_str(&link_text);
                    i = paren_end + 1;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Spawns the credits screen UI.
pub(super) fn setup(mut commands: Commands) {
    let content = spawn_page_container(&mut commands, OnCreditsScreen, false, Overflow::clip());

    // Strip markdown links and the "Detailed credits: credits.csv" line for button replacement
    let cleaned = strip_markdown_links(CREDITS_TEXT);
    let display_text = cleaned.replace("Detailed credits: credits.csv\n", "");

    commands.entity(content).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Credits"),
            TextFont::from_font_size(48.0),
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Scrollable credits content
        parent
            .spawn((
                Node {
                    width: Val::Percent(90.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                ScrollableCreditsContainer,
            ))
            .with_children(|scroll| {
                scroll
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    })
                    .with_children(|inner| {
                        inner.spawn((
                            Text::new(display_text),
                            TextFont::from_font_size(16.0),
                            TextColor(TEXT_COLOR),
                        ));
                    });
            });

        // View Sprite Credits button
        spawn_button(
            parent,
            "View Sprite Credits",
            SpriteCreditsButton,
            &SPRITE_CREDITS_BUTTON_STYLE,
        );

        // Back button
        spawn_button(parent, "Back", BackButton, &BACK_BUTTON_STYLE);
    });
}

/// Handles sprite credits button — logs a message pointing to the CSV file.
pub(super) fn handle_sprite_credits_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SpriteCreditsButton>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            info!("Sprite credits: see SPRITE_CREDITS.csv in the game directory");
        }
    }
}
