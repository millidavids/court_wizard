use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{CurrentLevel, KillStats};
use crate::state::AppState;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;

/// Sets up the wizard tower screen UI.
pub(super) fn setup_wizard_tower_screen(mut commands: Commands, current_level: Res<CurrentLevel>) {
    // Root container (fullscreen, centered column layout)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Wizard's Tower"),
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TITLE_COLOR),
            ));

            // Current level display
            parent.spawn((
                Text::new(format!("Level {}", current_level.0)),
                TextFont {
                    font_size: LEVEL_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Start Next Battle button
            spawn_button(
                parent,
                "Start Next Battle",
                WizardTowerButtonAction::StartNextBattle,
                &BUTTON_STYLE,
            );

            // Return to Menu button
            spawn_button(
                parent,
                "Return to Menu",
                WizardTowerButtonAction::ReturnToMenu,
                &BUTTON_STYLE,
            );
        });
}

/// Handles button actions on the wizard tower screen.
pub(super) fn handle_button_actions(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                WizardTowerButtonAction::StartNextBattle => {
                    kill_stats.reset();
                    next_app_state.set(AppState::Loading);
                }
                WizardTowerButtonAction::ReturnToMenu => {
                    kill_stats.reset();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

/// Cleans up the wizard tower screen UI.
pub(super) fn cleanup_wizard_tower_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnWizardTowerScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
