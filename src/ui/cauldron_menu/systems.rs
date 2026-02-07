use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::cauldron::brews::Brew;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::cauldron::messages::{CancelBrewMessage, StartBrewMessage};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseClicked;
use crate::state::InGameState;
use crate::ui::systems::spawn_button;

/// Spawns the cauldron menu UI when entering the CauldronMenu state.
pub(super) fn spawn_cauldron_menu_ui(
    mut commands: Commands,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());
    let active_brew = cauldron_query
        .single()
        .ok()
        .and_then(|state| state.active_brew());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnCauldronMenuScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Cauldron Brews"),
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Brewing status banner
            if let Some(brew) = active_brew {
                parent.spawn((
                    Text::new(format!("Currently brewing: {}", brew.name())),
                    TextFont {
                        font_size: BREWING_STATUS_FONT_SIZE,
                        ..default()
                    },
                    TextColor(BREWING_STATUS_COLOR),
                ));
            }

            // Brew cards container
            parent
                .spawn((
                    Node {
                        border: UiRect::all(Val::Px(FRAME_BORDER_WIDTH)),
                        padding: UiRect::all(Val::Px(FRAME_PADDING)),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(BREW_COLUMN_GAP),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    BorderColor::all(FRAME_BORDER_COLOR),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(FRAME_BACKGROUND),
                ))
                .with_children(|container| {
                    for brew in Brew::all() {
                        spawn_brew_card(container, *brew, is_brewing);
                    }
                });

            // Cancel brew button (only when brewing)
            if is_brewing {
                spawn_button(
                    parent,
                    "Cancel Brew",
                    CauldronMenuButtonAction::CancelBrew,
                    &CANCEL_BUTTON_STYLE,
                );
            }

            // Close button
            spawn_button(
                parent,
                "Close",
                CauldronMenuButtonAction::Close,
                &CLOSE_BUTTON_STYLE,
            );
        });
}

/// Spawns a single brew card with button, brew time, and description.
fn spawn_brew_card(parent: &mut ChildSpawnerCommands, brew: Brew, disabled: bool) {
    let (button_style, text_color, time_color) = if disabled {
        (
            &DISABLED_BUTTON_STYLE,
            DISABLED_TEXT_COLOR,
            DISABLED_BREW_TIME_COLOR,
        )
    } else {
        (&BUTTON_STYLE, TEXT_COLOR, BREW_TIME_COLOR)
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            width: Val::Px(BREW_COLUMN_WIDTH),
            padding: UiRect::horizontal(Val::Px(COLUMN_PADDING)),
            ..default()
        })
        .with_children(|card| {
            // Brew button (disabled buttons still get spawned but won't trigger action)
            spawn_button(
                card,
                brew.name(),
                CauldronMenuButtonAction::SelectBrew(brew),
                button_style,
            );

            // Brew time info
            card.spawn((
                Text::new(format!(
                    "Brew: {:.0}s | Buff: {:.0}s",
                    brew.brew_time(),
                    brew.buff_duration()
                )),
                TextFont {
                    font_size: BREW_TIME_FONT_SIZE,
                    ..default()
                },
                TextColor(time_color),
                TextLayout::new_with_justify(Justify::Center),
            ));

            // Description
            card.spawn((
                Text::new(brew.description()),
                TextFont {
                    font_size: DESCRIPTION_FONT_SIZE,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Handles button click actions — selects a brew, cancels, or closes the menu.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&CauldronMenuButtonAction>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut start_brew: MessageWriter<StartBrewMessage>,
    mut cancel_brew: MessageWriter<CancelBrewMessage>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                CauldronMenuButtonAction::SelectBrew(brew) => {
                    if !is_brewing {
                        start_brew.write(StartBrewMessage { brew: *brew });
                        next_in_game_state.set(InGameState::Running);
                    }
                }
                CauldronMenuButtonAction::CancelBrew => {
                    cancel_brew.write(CancelBrewMessage);
                    next_in_game_state.set(InGameState::Running);
                }
                CauldronMenuButtonAction::Close => {
                    next_in_game_state.set(InGameState::Running);
                }
            }
        }
    }
}

/// Handles keyboard input (ESC to close).
pub(super) fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Running);
    }
}

/// Despawns cauldron menu UI when exiting the CauldronMenu state.
pub(super) fn despawn_cauldron_menu_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnCauldronMenuScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Consumes the mouse button when exiting cauldron menu to prevent click bleed-through.
pub(super) fn consume_mouse_on_exit(mut mouse_state: ResMut<MouseButtonState>) {
    mouse_state.left_consumed = true;
}
