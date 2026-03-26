//! Systems for changelog screen.

use bevy::prelude::*;

use super::components::{OnChangelogScreen, ScrollableChangelogContainer};
use crate::ui::components::BackButton;
use crate::ui::main_menu::landing::constants::{BACK_BUTTON_STYLE, TEXT_COLOR};
use crate::ui::systems::{spawn_button, spawn_page_container, spawn_title_with_shadow};

const CHANGELOG_TEXT: &str = include_str!("../../../../CHANGELOG.md");

/// Spawns the changelog screen UI.
pub(super) fn setup(mut commands: Commands) {
    let content = spawn_page_container(&mut commands, OnChangelogScreen, false, crate::ui::systems::default_content_node());

    commands.entity(content).with_children(|parent| {
        // Title
        spawn_title_with_shadow(parent, "Changelog", 48.0, TEXT_COLOR, Node {
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        });

        // Scrollable changelog content
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
                ScrollableChangelogContainer,
            ))
            .with_children(|scroll| {
                scroll
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    })
                    .with_children(|content| {
                        content.spawn((
                            Text::new(CHANGELOG_TEXT),
                            TextFont::from_font_size(16.0),
                            TextColor(TEXT_COLOR),
                        ));
                    });
            });

        // Back button
        spawn_button(parent, "Back", BackButton, &BACK_BUTTON_STYLE);
    });
}
