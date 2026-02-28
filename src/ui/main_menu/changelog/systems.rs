//! Systems for changelog screen.

use bevy::ecs::relationship::Relationship;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use super::components::{BackButton, OnChangelogScreen, ScrollableChangelogContainer};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::MenuState;
use crate::ui::components::ButtonColors;
use crate::ui::main_menu::landing::constants::TEXT_COLOR;
use crate::ui::systems::spawn_page_container;

const BUTTON_COLOR: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
const BUTTON_BORDER_COLOR: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

const CHANGELOG_TEXT: &str = include_str!("../../../../CHANGELOG.md");

/// Spawns the changelog screen UI.
pub(super) fn setup(mut commands: Commands) {
    let content = spawn_page_container(
        &mut commands,
        OnChangelogScreen,
        false,
        Overflow::clip(),
    );

    commands.entity(content).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Changelog"),
            TextFont::from_font_size(48.0),
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Scrollable changelog content
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

/// Handles back button interactions.
pub(super) fn handle_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

/// Despawns all changelog screen entities.
pub fn cleanup(mut commands: Commands, query: Query<Entity, With<OnChangelogScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles mouse wheel scrolling for the changelog container.
pub fn handle_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<
        (&mut ScrollPosition, &ComputedNode),
        With<ScrollableChangelogContainer>,
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
