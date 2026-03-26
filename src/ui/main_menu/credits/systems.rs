//! Systems for credits screen.

use bevy::prelude::*;

use super::components::{OnCreditsScreen, ScrollableCreditsContainer};
use crate::ui::components::BackButton;
use crate::ui::main_menu::landing::constants::{BACK_BUTTON_STYLE, TEXT_COLOR};
use crate::ui::systems::{spawn_page_container, spawn_title_with_shadow};

const CREDITS_TEXT: &str = include_str!("../../../../CREDITS.md");

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

    let display_text = strip_markdown_links(CREDITS_TEXT);

    commands.entity(content).with_children(|parent| {
        // Title
        spawn_title_with_shadow(parent, "Credits", 48.0, TEXT_COLOR, Node {
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        });

        // Scrollable credits content
        parent
            .spawn((
                Node {
                    width: Val::Percent(90.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                crate::ui::systems::scroll_area_style(),
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

        // Back button
        crate::ui::systems::spawn_button(parent, "Back", BackButton, &BACK_BUTTON_STYLE);
    });
}
