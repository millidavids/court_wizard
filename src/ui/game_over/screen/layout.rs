use bevy::prelude::*;

use crate::config::save_data::load_unified_save;
use crate::config::{GameConfig, WizardType};
use crate::game::constants::INITIAL_DEFENDER_COUNT;
use crate::game::game_mode::components::{
    GameMode, ROGUELITE_MAX_LEVEL, RogueliteRunState, RunAggregateStats, format_time,
};
use crate::game::resources::{BattleInsightData, CurrentLevel, GameOutcome, KillStats};
use crate::game::time_travel::TimeTravelState;
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::game::units::wizard::archetypes::psychopath::constants::DEFENDER_KILL_THRESHOLD;
use crate::ui::systems::{spawn_button, spawn_page_container, spawn_title_with_shadow};

use super::super::components::*;
use super::super::constants::*;

/// Spawns the game-over / score screen UI.
///
/// Runs on `OnEnter(InGameState::ScoreScreen)` (single-player only). Multiplayer has its own
/// score screen (`setup_mp_score_screen`). Displays outcome (victory/defeat), kill stats,
/// insight earned, and seed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn setup_game_over_screen(
    mut commands: Commands,
    game_outcome: Res<GameOutcome>,
    kill_stats: Res<KillStats>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    battle_insight: Res<BattleInsightData>,
    time_travel: Option<Res<TimeTravelState>>,
    game_mode: Option<Res<GameMode>>,
    roguelite_run: Option<Res<RogueliteRunState>>,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
) {
    let is_time_travel = time_travel.is_some();
    let is_roguelite = matches!(game_mode.as_deref(), Some(&GameMode::Roguelite));
    let is_roguelite_run_end =
        is_roguelite && (game_outcome.is_defeat() || current_level.0 >= ROGUELITE_MAX_LEVEL);
    // Load lifetime stats (already accumulated by send_battle_ended)
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

    // Page container (standard overlay with content box)
    let content = spawn_page_container(
        &mut commands,
        OnGameOverScreen,
        false,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(100.0),
            padding: UiRect::all(Val::Px(40.0)),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
            ..default()
        },
    );

    // Trap gamepad focus to the score-screen buttons. Without this marker
    // the focus system keeps whatever focus target was active behind the
    // overlay (usually the in-game HUD buttons), so controller nav would
    // interact with the gameplay screen through the score screen.
    commands
        .entity(content)
        .insert(crate::ui::focus::ModalOverlay);
    commands.entity(content).with_children(|parent| {
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
                let title_text = if game_outcome.is_defeat() {
                    "DEFEAT"
                } else {
                    "VICTORY"
                };

                spawn_title_with_shadow(buttons, title_text, 60.0, TITLE_COLOR, Node::default());

                // Replays deliberately award no level progression, efficiency, or
                // permanent structures — without saying so, a "VICTORY" here reads
                // as an unlock that never arrives. Insight and lifetime stats *are*
                // still earned (see `send_battle_ended`), so the wording is limited
                // to level progress rather than claiming nothing was recorded.
                if is_time_travel {
                    buttons.spawn((
                        Text::new("Time Travel — level progress not recorded"),
                        TextFont::from_font_size(16.0),
                        TextColor(TEXT_COLOR),
                    ));
                }

                // Subtext for King death
                if *game_outcome == GameOutcome::DefeatKingDied {
                    buttons.spawn((
                        Text::new("The King died!"),
                        TextFont::from_font_size(24.0),
                        TextColor(TEXT_COLOR),
                    ));
                }

                // Subtext for not enough carnage (Psychopath)
                if *game_outcome == GameOutcome::DefeatNotEnoughCarnage {
                    let kill_pct = kill_stats.defenders_killed as f32 / total_defenders * 100.0;
                    let required_pct = DEFENDER_KILL_THRESHOLD * 100.0;
                    buttons.spawn((
                        Text::new(format!(
                            "Not enough carnage! Only {:.0}% killed ({:.0}% required)",
                            kill_pct, required_pct
                        )),
                        TextFont::from_font_size(24.0),
                        TextColor(TEXT_COLOR),
                    ));
                }

                if is_roguelite_run_end {
                    // Roguelite run ended (defeat or completed all levels)
                    spawn_button(
                        buttons,
                        "End Run",
                        GameOverButtonAction::EndRun,
                        &BUTTON_STYLE,
                    );
                } else if is_roguelite {
                    // Roguelite mid-run victory — jump straight to the next
                    // level, or detour to the tower to spend insight.
                    spawn_button(
                        buttons,
                        "Next Level",
                        GameOverButtonAction::NextLevel,
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Wizard Tower",
                        GameOverButtonAction::ReturnToTower,
                        &BUTTON_STYLE,
                    );
                } else if is_time_travel {
                    // Time travel: victory shows only "Wizard Tower"
                    // Defeat shows retry + wizard tower
                    if game_outcome.is_defeat() {
                        spawn_button(
                            buttons,
                            &format!("Time Rewind (Level {})", current_level.0),
                            GameOverButtonAction::PlayAgain,
                            &BUTTON_STYLE,
                        );
                    }

                    spawn_button(
                        buttons,
                        "Wizard Tower",
                        GameOverButtonAction::ReturnToTower,
                        &BUTTON_STYLE,
                    );
                } else {
                    // Normal Endless flow
                    if game_outcome.is_defeat() {
                        // Retry the same level, or head back to the tower.
                        spawn_button(
                            buttons,
                            &format!("Time Rewind (Level {})", current_level.0),
                            GameOverButtonAction::PlayAgain,
                            &BUTTON_STYLE,
                        );
                        spawn_button(
                            buttons,
                            "Wizard Tower",
                            GameOverButtonAction::ReturnToTower,
                            &BUTTON_STYLE,
                        );
                    } else {
                        // Victory — jump straight to the next level, or detour
                        // to the tower to spend insight.
                        spawn_button(
                            buttons,
                            "Next Level",
                            GameOverButtonAction::NextLevel,
                            &BUTTON_STYLE,
                        );
                        spawn_button(
                            buttons,
                            "Wizard Tower",
                            GameOverButtonAction::ReturnToTower,
                            &BUTTON_STYLE,
                        );
                    }

                    // Return to Menu button (both outcomes)
                    spawn_button(
                        buttons,
                        "Return to Menu",
                        GameOverButtonAction::ReturnToMenu,
                        &BUTTON_STYLE,
                    );
                }
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
                    TextFont::from_font_size(28.0),
                    TextColor(TITLE_COLOR),
                ));

                // Kill Statistics header
                stats.spawn((
                    Text::new("Kill Statistics:"),
                    TextFont::from_font_size(24.0),
                    TextColor(TEXT_COLOR),
                ));

                stats.spawn((
                    Text::new(format!("  Defenders Lost: {}", kill_stats.defenders_killed)),
                    TextFont::from_font_size(20.0),
                    TextColor(TEXT_COLOR),
                ));

                stats.spawn((
                    Text::new(format!(
                        "  Attackers Killed: {}",
                        kill_stats.attackers_killed
                    )),
                    TextFont::from_font_size(20.0),
                    TextColor(TEXT_COLOR),
                ));

                stats.spawn((
                    Text::new(format!("  Undead Killed: {}", kill_stats.undead_killed)),
                    TextFont::from_font_size(20.0),
                    TextColor(TEXT_COLOR),
                ));

                // Current efficiency
                stats.spawn((
                    Text::new(format!("  Efficiency: {:.1}%", current_efficiency)),
                    TextFont::from_font_size(20.0),
                    TextColor(TEXT_COLOR),
                ));

                // Carnage meter (Psychopath only)
                if config.wizard_type == WizardType::Psychopath {
                    let carnage_pct =
                        (kill_stats.defenders_killed as f32 / total_defenders * 100.0).min(100.0);
                    let carnage_color = if carnage_pct >= DEFENDER_KILL_THRESHOLD * 100.0 {
                        CARNAGE_MET_COLOR
                    } else {
                        CARNAGE_UNMET_COLOR
                    };
                    stats.spawn((
                        Text::new(format!(
                            "  Carnage: {:.1}% / {:.0}%",
                            carnage_pct,
                            DEFENDER_KILL_THRESHOLD * 100.0
                        )),
                        TextFont::from_font_size(20.0),
                        TextColor(carnage_color),
                    ));
                }

                // Insight earned this battle
                stats.spawn((
                    Text::new(format!(
                        "  Insight Earned: +{}",
                        battle_insight.insight_earned
                    )),
                    TextFont::from_font_size(20.0),
                    TextColor(INSIGHT_COLOR),
                ));

                if !is_roguelite {
                    // Past victory efficiency for current level (if exists)
                    if let Some(past_efficiency) =
                        config.efficiency_ratios.get(&current_level.0.to_string())
                    {
                        stats.spawn((
                            Text::new("Past Victory:"),
                            TextFont::from_font_size(24.0),
                            TextColor(TEXT_COLOR),
                        ));

                        stats.spawn((
                            Text::new(format!(
                                "  Level {}: {:.1}%",
                                current_level.0,
                                past_efficiency * 100.0
                            )),
                            TextFont::from_font_size(18.0),
                            TextColor(TEXT_COLOR),
                        ));
                    }

                    // Lifetime stats
                    stats.spawn((
                        Text::new("Lifetime:"),
                        TextFont::from_font_size(24.0),
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Attackers Killed: {}", lifetime_attackers)),
                        TextFont::from_font_size(20.0),
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Defenders Lost: {}", lifetime_defenders)),
                        TextFont::from_font_size(20.0),
                        TextColor(TEXT_COLOR),
                    ));

                    stats.spawn((
                        Text::new(format!("  Undead Killed: {}", lifetime_undead)),
                        TextFont::from_font_size(20.0),
                        TextColor(TEXT_COLOR),
                    ));
                }

                // Roguelite run summary (on run end only)
                if is_roguelite_run_end && let Some(ref run) = roguelite_run {
                    stats.spawn((
                        Text::new("Run Summary:"),
                        TextFont::from_font_size(24.0),
                        TextColor(TITLE_COLOR),
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    let agg = RunAggregateStats::from_level_stats(&run.level_stats);

                    stats.spawn((
                        Text::new(format!("  Levels Completed: {}", run.level_stats.len())),
                        TextFont::from_font_size(18.0),
                        TextColor(TEXT_COLOR),
                    ));
                    stats.spawn((
                        Text::new(format!("  Total Kills: {}", agg.total_kills)),
                        TextFont::from_font_size(18.0),
                        TextColor(TEXT_COLOR),
                    ));
                    stats.spawn((
                        Text::new(format!(
                            "  Avg Efficiency: {:.1}%",
                            agg.avg_efficiency * 100.0
                        )),
                        TextFont::from_font_size(18.0),
                        TextColor(TEXT_COLOR),
                    ));
                    stats.spawn((
                        Text::new(format!("  Total Time: {}", format_time(agg.total_time))),
                        TextFont::from_font_size(18.0),
                        TextColor(TEXT_COLOR),
                    ));

                    // Per-level breakdown (scrollable)
                    stats.spawn((
                        Text::new("  Per Level:"),
                        TextFont::from_font_size(16.0),
                        TextColor(INSIGHT_COLOR),
                    ));

                    stats
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                max_height: Val::Px(150.0),
                                overflow: Overflow::scroll_y(),
                                row_gap: Val::Px(2.0),
                                padding: UiRect::left(Val::Px(20.0)),
                                ..default()
                            },
                            ScrollPosition::default(),
                            crate::ui::focus::GamepadScrollTarget,
                        ))
                        .with_children(|scroll| {
                            for level_stat in &run.level_stats {
                                scroll.spawn((
                                    Text::new(format!(
                                        "Lv{}: {:.0}% | {} kills | {}",
                                        level_stat.level,
                                        level_stat.efficiency * 100.0,
                                        level_stat.total_kills(),
                                        format_time(level_stat.elapsed_time),
                                    )),
                                    TextFont::from_font_size(12.0),
                                    TextColor(TEXT_COLOR),
                                ));
                            }
                        });
                }
            });

        // Seed display (small text at the bottom)
        if let Some(ref seed) = game_seed {
            parent.spawn((
                Text::new(format!("Seed: {}", seed.0)),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    right: Val::Px(20.0),
                    ..default()
                },
            ));
        }
    });
}
