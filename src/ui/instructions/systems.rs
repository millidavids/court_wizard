//! Systems for instructions screen.

use bevy::ecs::relationship::Relationship;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use super::components::{BackButton, OnInstructionsScreen, ScrollableInstructionsContainer};
use super::constants::INSTRUCTIONS_TEXT;

// UI colors for instructions screen
const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);
const BUTTON_COLOR: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
const BUTTON_HOVER_COLOR: Color = Color::hsla(0.0, 0.0, 0.25, 1.0);

/// Spawns the instructions screen UI.
/// Uses a transparent background suitable for overlaying on pause menu.
pub(super) fn setup(mut commands: Commands, transparent_bg: bool) {
    let background_color = if transparent_bg {
        Color::srgba(0.0, 0.0, 0.0, 0.9)
    } else {
        Color::BLACK
    };

    let mut entity_commands = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(background_color),
        OnInstructionsScreen,
    ));

    // Add GlobalZIndex for pause menu to ensure it's above the game
    if transparent_bg {
        entity_commands.insert(GlobalZIndex(500));
    }

    entity_commands.with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Instructions"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
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
                    height: Val::Percent(70.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                ScrollableInstructionsContainer,
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            ))
            .with_children(|parent| {
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(INSTRUCTIONS_TEXT),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            TextLayout::new_with_linebreak(bevy::text::LineBreak::WordBoundary),
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
                BorderColor::all(Color::hsla(0.0, 0.0, 0.3, 1.0)),
                BorderRadius::all(Val::Px(8.0)),
                BackgroundColor(BUTTON_COLOR),
                BackButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Back"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                ));
            });
    });
}

/// Spawns instructions with solid black background (for main menu).
pub(super) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

/// Spawns instructions with transparent background (for pause menu).
pub(super) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}

/// Updates button colors on hover.
pub(super) fn update_button_colors(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    const NORMAL_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);
    const HOVER_BORDER: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);
    const PRESSED_BORDER: Color = Color::hsla(0.0, 0.0, 0.5, 1.0);

    for (interaction, mut bg_color, mut border_color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = Color::hsla(0.0, 0.0, 0.35, 1.0).into();
                *border_color = BorderColor::all(PRESSED_BORDER);
            }
            Interaction::Hovered => {
                *bg_color = BUTTON_HOVER_COLOR.into();
                *border_color = BorderColor::all(HOVER_BORDER);
            }
            Interaction::None => {
                *bg_color = BUTTON_COLOR.into();
                *border_color = BorderColor::all(NORMAL_BORDER);
            }
        }
    }
}

/// Despawns all instructions screen entities.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<OnInstructionsScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles mouse wheel scrolling for the instructions container.
pub(super) fn handle_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<
        (&mut ScrollPosition, &ComputedNode),
        With<ScrollableInstructionsContainer>,
    >,
    parent_query: Query<&ChildOf>,
) {
    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            bevy::input::mouse::MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        // Check if we're hovering over the scrollable container or any of its children
        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                // Walk up the hierarchy to find a scrollable container
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        break;
                    }

                    // Move to parent
                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
