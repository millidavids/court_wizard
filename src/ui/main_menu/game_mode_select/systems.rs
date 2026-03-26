use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::GameMode;
use crate::game::input::messages::MouseClicked;
use crate::state::MenuState;
use crate::ui::components::ButtonColors;
use crate::ui::systems::{scale_font_by_text_width, spawn_button, spawn_title_with_shadow};

use super::components::{
    DisabledModeButton, GameModeButtonAction, OnGameModeSelectScreen,
};
use super::constants::*;

/// Sets up the game mode selection screen UI.
pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            OnGameModeSelectScreen,
        ))
        .with_children(|parent| {
            // Title
            spawn_title_with_shadow(parent, "Choose Your Path", TITLE_FONT_SIZE, TEXT_COLOR, Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            });

            // 2x2 button grid
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(GRID_GAP),
                    ..default()
                })
                .with_children(|grid| {
                    // Top row: Story, Roguelite
                    grid.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(GRID_GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_disabled_button(
                            row,
                            "Story",
                            GameModeButtonAction::Story,
                            "Coming Soon",
                        );
                        spawn_mode_button(
                            row,
                            "Roguelite",
                            GameModeButtonAction::Roguelite,
                            "Fixed 25-level run",
                        );
                    });

                    // Bottom row: Endless, Multiplayer
                    grid.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(GRID_GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_mode_button(
                            row,
                            "Endless",
                            GameModeButtonAction::Endless,
                            "Infinite scaling",
                        );
                        spawn_disabled_button(
                            row,
                            "Multiplayer",
                            GameModeButtonAction::Multiplayer,
                            "Coming Soon",
                        );
                    });
                });

            // Back button
            spawn_button(
                parent,
                "Back",
                GameModeButtonAction::Back,
                &BACK_BUTTON_STYLE,
            );
        });
}

/// Spawns an active mode button with a subtitle description.
fn spawn_mode_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: GameModeButtonAction,
    subtitle: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(MODE_BUTTON_STYLE.width),
                height: Val::Px(MODE_BUTTON_STYLE.height),
                border: UiRect::all(Val::Px(MODE_BUTTON_STYLE.border_width)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BorderColor::all(MODE_BUTTON_STYLE.border),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(MODE_BUTTON_STYLE.background),
            ButtonColors {
                background: MODE_BUTTON_STYLE.background,
                border: MODE_BUTTON_STYLE.border,
            },
            action,
        ))
        .with_children(|button| {
            let font_size = scale_font_by_text_width(
                label.len() as f32,
                8.0,
                12.0,
                0.7,
                MODE_BUTTON_STYLE.font_size,
            );
            button.spawn((
                Text::new(label),
                TextFont::from_font_size(font_size),
                TextColor(MODE_BUTTON_STYLE.text_color),
            ));
            button.spawn((
                Text::new(subtitle),
                TextFont::from_font_size(SUBTITLE_FONT_SIZE),
                TextColor(SUBTITLE_COLOR),
            ));
        });
}

/// Spawns a disabled (grayed out) mode button that shows a toast on hover.
fn spawn_disabled_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: GameModeButtonAction,
    _subtitle: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(DISABLED_BUTTON_STYLE.width),
                height: Val::Px(DISABLED_BUTTON_STYLE.height),
                border: UiRect::all(Val::Px(DISABLED_BUTTON_STYLE.border_width)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BorderColor::all(DISABLED_BUTTON_STYLE.border),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(DISABLED_BUTTON_STYLE.background),
            ButtonColors {
                background: DISABLED_BUTTON_STYLE.background,
                border: DISABLED_BUTTON_STYLE.border,
            },
            action,
            DisabledModeButton,
        ))
        .with_children(|button| {
            let font_size = scale_font_by_text_width(
                label.len() as f32,
                8.0,
                12.0,
                0.7,
                DISABLED_BUTTON_STYLE.font_size,
            );
            button.spawn((
                Text::new(label),
                TextFont::from_font_size(font_size),
                TextColor(DISABLED_BUTTON_STYLE.text_color),
            ));
            button.spawn((
                Text::new("Coming Soon"),
                TextFont::from_font_size(SUBTITLE_FONT_SIZE),
                TextColor(SUBTITLE_COLOR),
            ));
        });
}

/// Handles clicks on active mode buttons.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&GameModeButtonAction, Without<DisabledModeButton>>,
    mut commands: Commands,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            match action {
                GameModeButtonAction::Roguelite => {
                    commands.insert_resource(GameMode::Roguelite);
                    next_menu_state.set(MenuState::WizardSelect);
                }
                GameModeButtonAction::Endless => {
                    commands.insert_resource(GameMode::Endless);
                    next_menu_state.set(MenuState::WizardSelect);
                }
                GameModeButtonAction::Back => {
                    next_menu_state.set(MenuState::Landing);
                }
                // Story and Multiplayer are filtered out by Without<DisabledModeButton>
                _ => {}
            }
        }
    }
}

