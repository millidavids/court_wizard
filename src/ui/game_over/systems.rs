use bevy::prelude::*;

use crate::config::{ConfigChanged, GameConfig};
use crate::game::constants::INITIAL_DEFENDER_COUNT;
use crate::game::resources::{CurrentLevel, GameOutcome, KillStats};
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::state::{AppState, InGameState};
use crate::ui::systems::spawn_button;

use super::components::*;
use super::styles::*;

/// Saves efficiency for current level to config when entering game over screen.
///
/// This system runs on OnEnter(InGameState::GameOver) BEFORE setup_game_over_screen
/// to save efficiency, but DOES NOT update the level yet (that happens after UI displays).
pub fn save_efficiency_to_config(
    current_level: Res<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    kill_stats: Res<KillStats>,
    mut config_events: MessageWriter<ConfigChanged>,
) {
    // Calculate efficiency ratio for this level
    let total_defenders = (INITIAL_DEFENDER_COUNT + INITIAL_ARCHER_DEFENDER_COUNT) as f32;
    let defenders_lost = kill_stats.defenders_killed as f32;
    let efficiency = 1.0 - (defenders_lost / total_defenders);

    // Store efficiency ratio for current level (the level that was just played)
    config
        .efficiency_ratios
        .insert(current_level.0.to_string(), efficiency);

    // Trigger config save immediately
    config_events.write(ConfigChanged);
}

/// Updates level and saves to config after game over screen is displayed.
///
/// This system runs AFTER setup_game_over_screen so the UI shows the correct
/// level that was just played, not the next level.
pub fn update_level_after_display(
    mut current_level: ResMut<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    game_outcome: Res<GameOutcome>,
    mut config_events: MessageWriter<ConfigChanged>,
) {
    // Update level based on win/loss
    match *game_outcome {
        GameOutcome::Victory => {
            current_level.0 += 1;
            // Update highest level if surpassed
            if current_level.0 > config.highest_level_achieved {
                config.highest_level_achieved = current_level.0;
            }
        }
        GameOutcome::Defeat | GameOutcome::DefeatKingDied => {
            // Drop one level, minimum 1
            current_level.0 = current_level.0.saturating_sub(1).max(1);
        }
    }

    // Save current level to config
    config.current_level = current_level.0;

    // Trigger config save immediately
    config_events.write(ConfigChanged);
}

pub fn setup_game_over_screen(
    mut commands: Commands,
    game_outcome: Res<GameOutcome>,
    kill_stats: Res<KillStats>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
) {
    // Calculate current efficiency
    let total_defenders = (INITIAL_DEFENDER_COUNT + INITIAL_ARCHER_DEFENDER_COUNT) as f32;
    let defenders_lost = kill_stats.defenders_killed as f32;
    let current_efficiency = (1.0 - (defenders_lost / total_defenders)) * 100.0;

    // Root container (fullscreen, horizontal layout)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(100.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnGameOverScreen,
        ))
        .with_children(|parent| {
            // Left column - Buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|buttons| {
                    // Victory/Defeat title
                    let title_text = match *game_outcome {
                        GameOutcome::Victory => "VICTORY",
                        GameOutcome::Defeat | GameOutcome::DefeatKingDied => "DEFEAT",
                    };

                    buttons.spawn((
                        Text::new(title_text),
                        TextFont {
                            font_size: 60.0,
                            ..default()
                        },
                        TextColor(TITLE_COLOR),
                    ));

                    // Subtext for King death
                    if *game_outcome == GameOutcome::DefeatKingDied {
                        buttons.spawn((
                            Text::new("The King died!"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    }

                    // Play Again button with level progression indicator
                    let button_text = match *game_outcome {
                        GameOutcome::Victory => {
                            format!("Advance to Level {}", current_level.0 + 1)
                        }
                        GameOutcome::Defeat | GameOutcome::DefeatKingDied => {
                            let next_level = current_level.0.saturating_sub(1).max(1);
                            if next_level < current_level.0 {
                                format!("Drop to Level {}", next_level)
                            } else {
                                format!("Stay at Level {}", next_level)
                            }
                        }
                    };

                    spawn_button(
                        buttons,
                        &button_text,
                        GameOverButtonAction::PlayAgain,
                        &BUTTON_STYLE,
                    );

                    // Return to Menu button
                    spawn_button(
                        buttons,
                        "Return to Menu",
                        GameOverButtonAction::ReturnToMenu,
                        &BUTTON_STYLE,
                    );
                });

            // Right column - Statistics
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|stats| {
                    // Current Level
                    stats.spawn((
                        Text::new(format!("Current Level: {}", current_level.0)),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(TITLE_COLOR),
                    ));

                    // Kill Statistics header
                    stats.spawn((
                        Text::new("Kill Statistics:"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Defenders Lost: {}", kill_stats.defenders_killed)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!(
                            "  Attackers Killed: {}",
                            kill_stats.attackers_killed
                        )),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Undead Killed: {}", kill_stats.undead_killed)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    // Current efficiency
                    stats.spawn((
                        Text::new(format!("  Efficiency: {:.1}%", current_efficiency)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    // Past victory efficiency for current level (if exists)
                    if let Some(past_efficiency) =
                        config.efficiency_ratios.get(&current_level.0.to_string())
                    {
                        stats.spawn((
                            Text::new("Past Victory:"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));

                        stats.spawn((
                            Text::new(format!(
                                "  Level {}: {:.1}%",
                                current_level.0,
                                past_efficiency * 100.0
                            )),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    }
                });
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
                    // (level was already updated and saved when entering GameOver state)
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
