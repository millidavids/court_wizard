use bevy::prelude::*;

use crate::config::save_data::{
    AchievementId, accumulate_kill_stats, get_total_levels_completed, increment_games_played,
    increment_levels_completed, load_unified_save, unlock_achievement,
};
use crate::config::{ActiveSave, ConfigChanged, GameConfig};
use crate::game::constants::INITIAL_DEFENDER_COUNT;
use crate::game::input::messages::MouseClicked;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::{
    AchievementTracker, CurrentLevel, GameOutcome, KillStats, RetryTracker,
};
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::state::{AppState, InGameState};
use crate::ui::systems::spawn_button;

use super::components::*;
use super::styles::*;

/// Helper to try unlocking an achievement, sending the popup message if newly unlocked.
fn try_unlock(
    id: AchievementId,
    tracker: &mut AchievementTracker,
    achievement_events: &mut MessageWriter<AchievementUnlockedMessage>,
) {
    if !tracker.unlocked.contains(id.id()) {
        tracker.unlocked.insert(id.id().to_string());
        unlock_achievement(id);
        achievement_events.write(AchievementUnlockedMessage { id });
    }
}

/// Updates meta-progression counters and checks all Victory & Progression achievements.
pub(super) fn check_victory_progression_achievements(
    game_outcome: Res<GameOutcome>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    kill_stats: Res<KillStats>,
    mut tracker: ResMut<AchievementTracker>,
    mut retry_tracker: ResMut<RetryTracker>,
    mut achievement_events: MessageWriter<AchievementUnlockedMessage>,
) {
    // Always increment games played and accumulate kill stats
    increment_games_played();
    accumulate_kill_stats(
        kill_stats.defenders_killed,
        kill_stats.attackers_killed,
        kill_stats.undead_killed,
    );

    let is_victory = *game_outcome == GameOutcome::Victory;

    if is_victory {
        // Increment total wins counter in save
        increment_levels_completed();

        // Reset retry tracker on victory (player advances to next level)
        retry_tracker.level = 0;
        retry_tracker.attempts = 0;

        // --- Win count achievements ---
        let total_wins = get_total_levels_completed();

        if total_wins >= 1 {
            try_unlock(
                AchievementId::FirstVictory,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if total_wins >= 5 {
            try_unlock(
                AchievementId::ApprenticeWizard,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if total_wins >= 10 {
            try_unlock(
                AchievementId::CourtWizard,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if total_wins >= 25 {
            try_unlock(
                AchievementId::Archmage,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if total_wins >= 50 {
            try_unlock(
                AchievementId::LegendsSpeakYourName,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if total_wins >= 100 {
            try_unlock(
                AchievementId::Immortalized,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if total_wins >= 200 {
            try_unlock(
                AchievementId::TheGrindNeverStops,
                &mut tracker,
                &mut achievement_events,
            );
        }
    } else {
        // Defeat — track retries for this level
        if retry_tracker.level == current_level.0 {
            retry_tracker.attempts += 1;
        } else {
            retry_tracker.level = current_level.0;
            retry_tracker.attempts = 1;
        }

        // --- Retry achievements ---
        if retry_tracker.attempts >= 5 {
            try_unlock(
                AchievementId::Stubborn,
                &mut tracker,
                &mut achievement_events,
            );
        }
        if retry_tracker.attempts >= 15 {
            try_unlock(
                AchievementId::ExtremelyStubborn,
                &mut tracker,
                &mut achievement_events,
            );
        }

        // --- Defeat & Failure achievements ---

        // Tactical Retreat: lose your first battle
        try_unlock(
            AchievementId::TacticalRetreat,
            &mut tracker,
            &mut achievement_events,
        );

        // The King is Dead: lose because the king died
        if *game_outcome == GameOutcome::DefeatKingDied {
            try_unlock(
                AchievementId::TheKingIsDead,
                &mut tracker,
                &mut achievement_events,
            );
        }

        // Total Wipe: lose with zero defenders remaining (Defeat means all defenders dead)
        if *game_outcome == GameOutcome::Defeat {
            try_unlock(
                AchievementId::TotalWipe,
                &mut tracker,
                &mut achievement_events,
            );
        }

        // Speedrun (Wrong Direction): lose in under 30 seconds
        if kill_stats.elapsed_time < 30.0 {
            try_unlock(
                AchievementId::SpeedrunWrongDirection,
                &mut tracker,
                &mut achievement_events,
            );
        }

        // Pyrrhic Defeat: kill 90%+ of attackers but still lose
        if kill_stats.total_attackers_spawned > 0 {
            let kill_ratio =
                kill_stats.attackers_killed as f32 / kill_stats.total_attackers_spawned as f32;
            if kill_ratio >= 0.9 {
                try_unlock(
                    AchievementId::PyrrhicDefeat,
                    &mut tracker,
                    &mut achievement_events,
                );
            }
        }

        // It Was Going So Well: lose after no defenders died for 2+ minutes
        if let Some(first_death_time) = kill_stats.first_defender_death_time {
            if first_death_time >= 120.0 {
                try_unlock(
                    AchievementId::ItWasGoingSoWell,
                    &mut tracker,
                    &mut achievement_events,
                );
            }
        }

        // Friendly Fire Department: kill 10+ defenders with spells in one battle
        if kill_stats.defenders_killed_by_spell >= 10 {
            try_unlock(
                AchievementId::FriendlyFireDepartment,
                &mut tracker,
                &mut achievement_events,
            );
        }

        // Accidental Regicide: kill the king with your own spell
        if kill_stats.king_killed_by_spell {
            try_unlock(
                AchievementId::AccidentalRegicide,
                &mut tracker,
                &mut achievement_events,
            );
        }
    }

    // --- Level-based achievements (checked on both victory and defeat) ---
    // On victory, the player will advance to current_level + 1 (update_level_after_display
    // runs later in the chain), so the effective highest reached is max of saved highest
    // and current_level + 1 on victory, or current_level on defeat.
    let effective_highest = if is_victory {
        config.highest_level_achieved.max(current_level.0 + 1)
    } else {
        config.highest_level_achieved
    };
    let highest = effective_highest;

    if highest >= 10 {
        try_unlock(
            AchievementId::OneMoreLevel,
            &mut tracker,
            &mut achievement_events,
        );
    }
    if highest >= 25 {
        try_unlock(
            AchievementId::IntoTheDeep,
            &mut tracker,
            &mut achievement_events,
        );
    }
    if highest >= 50 {
        try_unlock(
            AchievementId::Absurdity,
            &mut tracker,
            &mut achievement_events,
        );
    }
    if highest >= 100 {
        try_unlock(
            AchievementId::Level100,
            &mut tracker,
            &mut achievement_events,
        );
    }
}

/// Saves efficiency for current level to config when entering game over screen.
///
/// This system runs on OnEnter(InGameState::GameOver) BEFORE setup_game_over_screen
/// to save efficiency, but DOES NOT update the level yet (that happens after UI displays).
pub(super) fn save_efficiency_to_config(
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
pub(super) fn update_level_after_display(
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
            // Keep current level - player retries the same level
            // No change to current_level.0
        }
    }

    // Save current level to config
    config.current_level = current_level.0;

    // Trigger config save immediately
    config_events.write(ConfigChanged);
}

pub(super) fn setup_game_over_screen(
    mut commands: Commands,
    game_outcome: Res<GameOutcome>,
    kill_stats: Res<KillStats>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
) {
    // Load lifetime stats (already accumulated by check_victory_progression_achievements)
    let save = load_unified_save();
    let lifetime_attackers = save
        .as_ref()
        .map(|s| s.player.total_attackers_killed)
        .unwrap_or(0);
    let lifetime_defenders = save
        .as_ref()
        .map(|s| s.player.total_defenders_killed)
        .unwrap_or(0);
    let lifetime_undead = save
        .as_ref()
        .map(|s| s.player.total_undead_killed)
        .unwrap_or(0);
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
                            // font removed (using default),
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
                                // font removed (using default),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    }

                    // Play Again button text depends on outcome
                    let button_text = match *game_outcome {
                        GameOutcome::Victory => "Continue".to_string(),
                        GameOutcome::Defeat | GameOutcome::DefeatKingDied => {
                            format!("Try Again (Level {})", current_level.0)
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
                            // font removed (using default),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(TITLE_COLOR),
                    ));

                    // Kill Statistics header
                    stats.spawn((
                        Text::new("Kill Statistics:"),
                        TextFont {
                            // font removed (using default),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Defenders Lost: {}", kill_stats.defenders_killed)),
                        TextFont {
                            // font removed (using default),
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
                            // font removed (using default),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Undead Killed: {}", kill_stats.undead_killed)),
                        TextFont {
                            // font removed (using default),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    // Current efficiency
                    stats.spawn((
                        Text::new(format!("  Efficiency: {:.1}%", current_efficiency)),
                        TextFont {
                            // font removed (using default),
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

                    // Lifetime stats
                    stats.spawn((
                        Text::new("Lifetime:"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Attackers Killed: {}", lifetime_attackers)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Defenders Lost: {}", lifetime_defenders)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Undead Killed: {}", lifetime_undead)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
}

pub(super) fn handle_button_actions(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&GameOverButtonAction>,
    game_outcome: Res<GameOutcome>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                GameOverButtonAction::PlayAgain => {
                    // Victory: Go to Wizard Tower for progression
                    // Defeat: Immediate retry, skip tower
                    match *game_outcome {
                        GameOutcome::Victory => {
                            // Go to Wizard Tower (don't reset stats yet)
                            next_in_game_state.set(InGameState::WizardTower);
                        }
                        GameOutcome::Defeat | GameOutcome::DefeatKingDied => {
                            // Immediate retry: reset stats and reload
                            kill_stats.reset();
                            next_app_state.set(AppState::Loading);
                        }
                    }
                }
                GameOverButtonAction::ReturnToMenu => {
                    // Reset stats, clear active save, and go to main menu
                    kill_stats.reset();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

pub(super) fn cleanup_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnGameOverScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
