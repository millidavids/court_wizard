//! Systems for instructions screen.

use bevy::prelude::*;

use super::components::{BackButton, OnInstructionsScreen, ScrollableInstructionsContainer};
use super::constants::INSTRUCTIONS_TEXT;
use crate::ui::main_menu::BACK_BUTTON_STYLE;
use crate::ui::systems::spawn_title_with_shadow;

use crate::ui::constants::TEXT_PRIMARY;

const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Spawns the instructions screen UI.
pub(super) fn setup(mut commands: Commands, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

    let content = spawn_page_container(
        &mut commands,
        OnInstructionsScreen,
        pause_menu,
        crate::ui::systems::default_content_node(),
    );

    commands.entity(content).with_children(|parent| {
        // Header row: title left, Back button right
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            })
            .with_children(|header| {
                spawn_title_with_shadow(
                    header,
                    "Instructions",
                    48.0,
                    TEXT_COLOR,
                    Node::default(),
                );
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                crate::ui::systems::spawn_button(header, "Back", BackButton, &BACK_BUTTON_STYLE);
            });

        // Scrollable instructions content
        parent
            .spawn((
                Node {
                    width: Val::Percent(90.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                crate::ui::systems::scroll_area_style(),
                ScrollPosition::default(),
                ScrollableInstructionsContainer,
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
                            Text::new(INSTRUCTIONS_TEXT),
                            TextFont::from_font_size(16.0),
                            TextColor(TEXT_COLOR),
                            TextLayout::new_with_linebreak(bevy::text::LineBreak::WordBoundary),
                        ));
                    });
            });
    });
}

/// Spawns instructions for main menu.
pub(super) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

/// Spawns instructions for pause menu.
pub(super) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}
