//! Systems for changelog screen.

use bevy::prelude::*;

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

