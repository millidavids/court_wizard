use bevy::prelude::*;

use crate::game::resources::{GameOutcome, KillStats};
use crate::state::{AppState, InGameState};
use crate::ui::systems::spawn_button;

use super::components::*;
use super::styles::*;

pub fn setup_game_over_screen(
    mut commands: Commands,
    game_outcome: Res<GameOutcome>,
    kill_stats: Res<KillStats>,
) {
    // Root container (fullscreen, centered)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnGameOverScreen,
        ))
        .with_children(|parent| {
            // Victory/Defeat title
            let title_text = match *game_outcome {
                GameOutcome::Victory => "VICTORY",
                GameOutcome::Defeat => "DEFEAT",
            };

            parent.spawn((
                Text::new(title_text),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(TITLE_COLOR),
            ));

            // Statistics section
            parent.spawn((
                Text::new("Kill Statistics:"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            parent.spawn((
                Text::new(format!("Defenders Lost: {}", kill_stats.defenders_killed)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            parent.spawn((
                Text::new(format!("Attackers Killed: {}", kill_stats.attackers_killed)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            parent.spawn((
                Text::new(format!("Undead Killed: {}", kill_stats.undead_killed)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Play Again button
            spawn_button(
                parent,
                "Play Again",
                GameOverButtonAction::PlayAgain,
                &BUTTON_STYLE,
            );

            // Return to Menu button
            spawn_button(
                parent,
                "Return to Menu",
                GameOverButtonAction::ReturnToMenu,
                &BUTTON_STYLE,
            );
        });
}

pub fn handle_button_actions(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut kill_stats: ResMut<KillStats>,
    interaction_query: Query<
        (&Interaction, &GameOverButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                GameOverButtonAction::PlayAgain => {
                    // Reset stats and return to Running state
                    kill_stats.reset();
                    next_in_game_state.set(InGameState::Running);
                }
                GameOverButtonAction::ReturnToMenu => {
                    // Reset stats and go to main menu (exits InGame state)
                    kill_stats.reset();
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

pub fn cleanup_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnGameOverScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
