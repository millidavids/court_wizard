//! Pause menu main screen systems.

use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, InGameState, PauseMenuState};
use crate::ui::systems::{spawn_button, spawn_page_container, spawn_title_with_shadow};

use super::components::{OnPauseMainScreen, PauseMenuButtonAction};
use super::constants::{BUTTON_STYLE, MARGIN, TEXT_COLOR, TITLE_FONT_SIZE};

/// Sets up the pause menu main screen UI.
///
/// Spawns the root UI node containing the title and menu buttons.
/// All spawned entities are marked with `OnPauseMainScreen` for cleanup.
pub fn setup(
    mut commands: Commands,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
) {
    let content = spawn_page_container(
        &mut commands,
        OnPauseMainScreen,
        true,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(MARGIN),
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
            ..default()
        },
    );

    commands.entity(content).with_children(|parent| {
        // Title text
        spawn_title_with_shadow(parent, "Paused", TITLE_FONT_SIZE, TEXT_COLOR, Node {
            margin: UiRect::bottom(Val::Px(MARGIN * 2.0)),
            ..default()
        });

        // Continue button
        spawn_button(
            parent,
            "Continue",
            PauseMenuButtonAction::Continue,
            &BUTTON_STYLE,
        );

        // Settings button
        spawn_button(
            parent,
            "Settings",
            PauseMenuButtonAction::Settings,
            &BUTTON_STYLE,
        );

        // Instructions button
        spawn_button(
            parent,
            "Instructions",
            PauseMenuButtonAction::Instructions,
            &BUTTON_STYLE,
        );

        // Compendium button
        spawn_button(
            parent,
            "Compendium",
            PauseMenuButtonAction::Compendium,
            &BUTTON_STYLE,
        );

        // Exit button
        spawn_button(
            parent,
            "Exit to Menu",
            PauseMenuButtonAction::Exit,
            &BUTTON_STYLE,
        );

        // Seed display
        if let Some(ref seed) = game_seed {
            parent.spawn((
                Text::new(format!("Seed: {}", seed.0)),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                Node {
                    margin: UiRect::top(Val::Px(MARGIN * 2.0)),
                    ..default()
                },
            ));
        }
    });
}

/// Handles pause menu button actions.
///
/// Triggers state transitions based on the button's `PauseMenuButtonAction` component.
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&PauseMenuButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                PauseMenuButtonAction::Continue => {
                    next_in_game_state.set(InGameState::Running);
                }
                PauseMenuButtonAction::Settings => {
                    next_pause_menu_state.set(PauseMenuState::Settings);
                }
                PauseMenuButtonAction::Instructions => {
                    next_pause_menu_state.set(PauseMenuState::Instructions);
                }
                PauseMenuButtonAction::Compendium => {
                    next_pause_menu_state.set(PauseMenuState::Compendium);
                }
                PauseMenuButtonAction::Exit => {
                    channel_change.write(ChannelChangeMessage);
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}
