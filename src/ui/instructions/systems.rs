//! Systems for instructions screen.

use bevy::prelude::*;

use super::components::{BackButton, OnInstructionsScreen, ScrollableInstructionsContainer};
use super::constants::INSTRUCTIONS_TEXT;
use crate::ui::components::ButtonColors;

// UI colors for instructions screen
const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);
const BUTTON_COLOR: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
const BUTTON_BORDER_COLOR: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

/// Spawns the instructions screen UI.
pub(super) fn setup(mut commands: Commands, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

    let content = spawn_page_container(
        &mut commands,
        OnInstructionsScreen,
        pause_menu,
        Overflow::clip(),
    );

    commands.entity(content).with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Instructions"),
                TextFont::from_font_size(48.0),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Scrollable instructions content
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
                                TextLayout::new_with_linebreak(
                                    bevy::text::LineBreak::WordBoundary,
                                ),
                            ));
                        });
                });

            // Back button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(BUTTON_COLOR),
                    ButtonColors {
                        background: BUTTON_COLOR,
                        border: BUTTON_BORDER_COLOR,
                    },
                    BackButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Back"),
                        TextFont::from_font_size(32.0),
                        TextColor(TEXT_COLOR),
                    ));
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

/// Despawns all instructions screen entities.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnInstructionsScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

