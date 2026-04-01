use bevy::prelude::*;

use crate::config::save_data::{SavedCrystal, SavedWall, load_unified_save};
use crate::config::{ActiveSave, ConfigChanged, GameConfig, WizardType};
use crate::game::constants::INITIAL_DEFENDER_COUNT;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{
    GameMode, LevelRunStats, ROGUELITE_MAX_LEVEL, RogueliteRunState, RunAggregateStats,
    format_time, is_roguelite_mode,
};
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{
    BattleInsightData, CurrentLevel, GameOutcome, KillStats, TimeTravelState,
};
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::game::units::wizard::archetypes::psychopath::constants::DEFENDER_KILL_THRESHOLD;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::state::AppState;
use crate::ui::systems::{spawn_button, spawn_page_container, spawn_title_with_shadow};

use super::components::*;
use super::constants::*;

/// Saves efficiency for current level to config when entering game over screen.
///
/// This system runs on OnEnter(InGameState::ScoreScreen) BEFORE setup_game_over_screen
/// to save efficiency, but DOES NOT update the level yet (that happens after UI displays).
pub(super) fn save_efficiency_to_config(
    current_level: Res<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    kill_stats: Res<KillStats>,
    mut config_events: MessageWriter<ConfigChanged>,
    time_travel: Option<Res<TimeTravelState>>,
    game_mode: Option<Res<GameMode>>,
    game_outcome: Res<GameOutcome>,
) {
    if time_travel.is_some() {
        return;
    }
    // In Roguelite mode, efficiency is tracked in the run state, not in config
    if is_roguelite_mode(game_mode.as_deref()) {
        return;
    }
    // Defeat = 0% efficiency (king died)
    let efficiency = if game_outcome.is_defeat() {
        0.0
    } else {
        let total_defenders = (INITIAL_DEFENDER_COUNT + INITIAL_ARCHER_DEFENDER_COUNT) as f32;
        let defenders_lost = kill_stats.defenders_killed as f32;
        1.0 - (defenders_lost / total_defenders)
    };

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
    time_travel: Option<Res<TimeTravelState>>,
    game_mode: Option<Res<GameMode>>,
) {
    if time_travel.is_some() {
        return;
    }
    let is_roguelite = is_roguelite_mode(game_mode.as_deref());
    // Update level based on win/loss
    if *game_outcome == GameOutcome::Victory {
        current_level.0 += 1;
        // Update highest level if surpassed (Endless only)
        if !is_roguelite && current_level.0 > config.highest_level_achieved {
            config.highest_level_achieved = current_level.0;
        }
    }
    // Defeat: keep current level - player retries the same level

    // Save current level to config (for next level in run or normal progression)
    config.current_level = current_level.0;

    // Trigger config save immediately
    config_events.write(ConfigChanged);
}

/// Saves all living walls as permanent walls on victory.
pub(super) fn save_walls_on_victory(
    game_outcome: Res<GameOutcome>,
    mut config: ResMut<GameConfig>,
    walls: Query<&WallOfStone>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    if time_travel.is_some() || *game_outcome != GameOutcome::Victory {
        return;
    }

    let saved: Vec<SavedWall> = walls
        .iter()
        .map(|wall| SavedWall {
            center_x: wall.center.x,
            center_z: wall.center.z,
            half_length: wall.half_length,
            half_width: wall.half_width,
            forward_x: wall.forward.x,
            forward_z: wall.forward.z,
            height: wall.height,
            empowerment: wall.empowerment,
        })
        .collect();

    config.saved_walls = saved;
}

/// Saves all permanent crystals on victory so they persist to the next level.
pub(super) fn save_crystals_on_victory(
    game_outcome: Res<GameOutcome>,
    mut config: ResMut<GameConfig>,
    crystals: Query<&ArcaneCrystal>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    if time_travel.is_some() || *game_outcome != GameOutcome::Victory {
        return;
    }

    let saved: Vec<SavedCrystal> = crystals
        .iter()
        .filter(|c| c.permanent)
        .map(|c| SavedCrystal {
            x: c.position.x,
            z: c.position.z,
            range: c.range,
            empowerment: c.empowerment,
        })
        .collect();

    config.saved_crystals = saved;
}

/// Saves all living terrain on victory.
pub(super) fn save_terrain_on_victory(
    game_outcome: Res<GameOutcome>,
    mut config: ResMut<GameConfig>,
    trees: Query<&crate::game::terrain::tree::components::Tree>,
    ponds: Query<&crate::game::terrain::pond::components::Pond>,
    bushes: Query<
        &crate::game::terrain::bush::components::Bush,
        Without<crate::game::terrain::bush::components::BurningBush>,
    >,
    boulders: Query<&crate::game::terrain::boulder::components::Boulder>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    if time_travel.is_some() || *game_outcome != GameOutcome::Victory {
        return;
    }

    config.saved_trees = trees
        .iter()
        .map(|t| crate::config::save_data::SavedTree {
            x: t.center.x,
            z: t.center.z,
            scale: t.radius / crate::game::terrain::tree::constants::TREE_RADIUS,
        })
        .collect();

    config.saved_ponds = ponds
        .iter()
        .map(|p| crate::config::save_data::SavedPond {
            x: p.center.x,
            z: p.center.z,
            radius: p.radius,
        })
        .collect();

    config.saved_bushes = bushes
        .iter()
        .map(|b| crate::config::save_data::SavedBush {
            x: b.center.x,
            z: b.center.z,
            scale: b.radius / crate::game::terrain::bush::constants::BUSH_RADIUS,
        })
        .collect();

    // Save all living boulders (both terrain-placed and thrown)
    config.saved_boulders = boulders
        .iter()
        .filter(|b| !b.sinking)
        .map(|b| crate::config::save_data::SavedBoulder {
            x: b.center.x,
            z: b.center.z,
            scale: b.radius / crate::game::terrain::boulder::constants::ROCK_RADIUS,
        })
        .collect();
}

#[allow(clippy::too_many_arguments)]
pub(super) fn setup_game_over_screen(
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
                    // Roguelite mid-run victory — continue to next level
                    spawn_button(
                        buttons,
                        "Continue",
                        GameOverButtonAction::PlayAgain,
                        &BUTTON_STYLE,
                    );
                } else if is_time_travel {
                    // Time travel: victory shows only "Return to Tower"
                    // Defeat shows retry + return to tower
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
                        "Return to Tower",
                        GameOverButtonAction::ReturnToTower,
                        &BUTTON_STYLE,
                    );
                } else {
                    // Normal Endless flow
                    let button_text = if game_outcome.is_defeat() {
                        format!("Time Rewind (Level {})", current_level.0)
                    } else {
                        "Continue".to_string()
                    };

                    spawn_button(
                        buttons,
                        &button_text,
                        GameOverButtonAction::PlayAgain,
                        &BUTTON_STYLE,
                    );

                    // Return to Tower button (only on defeat)
                    if game_outcome.is_defeat() {
                        spawn_button(
                            buttons,
                            "Return to Tower",
                            GameOverButtonAction::ReturnToTower,
                            &BUTTON_STYLE,
                        );
                    }

                    // Return to Menu button
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
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            max_height: Val::Px(150.0),
                            overflow: Overflow::scroll_y(),
                            row_gap: Val::Px(2.0),
                            padding: UiRect::left(Val::Px(20.0)),
                            ..default()
                        })
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

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&GameOverButtonAction>,
    game_outcome: Res<GameOutcome>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    config: Res<GameConfig>,
    time_travel: Option<Res<TimeTravelState>>,
    roguelite_run: Option<Res<RogueliteRunState>>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            match action {
                GameOverButtonAction::PlayAgain => {
                    if let Some(ref tt) = time_travel {
                        if game_outcome.is_defeat() {
                            // Retry the time-traveled level (TimeTravelState persists)
                            kill_stats.reset();
                            next_app_state.set(AppState::Loading);
                        } else {
                            // Time travel victory: restore real level, return to tower
                            current_level.0 = tt.real_level;
                            commands.remove_resource::<TimeTravelState>();
                            next_app_state.set(AppState::MetaGame);
                        }
                    } else if game_outcome.is_defeat() {
                        kill_stats.reset();
                        next_app_state.set(AppState::Loading);
                    } else {
                        next_app_state.set(AppState::MetaGame);
                    }
                }
                GameOverButtonAction::ReturnToTower => {
                    kill_stats.reset();
                    if let Some(ref tt) = time_travel {
                        current_level.0 = tt.real_level;
                        commands.remove_resource::<TimeTravelState>();
                    }
                    next_app_state.set(AppState::MetaGame);
                }
                GameOverButtonAction::ReturnToMenu => {
                    kill_stats.reset();
                    if time_travel.is_some() {
                        commands.remove_resource::<TimeTravelState>();
                    }
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
                GameOverButtonAction::EndRun => {
                    // Save roguelite run to history
                    if let Some(ref run) = roguelite_run {
                        let roguelite_run_data = crate::config::save_data::RogueliteRun {
                            victory: !game_outcome.is_defeat(),
                            levels_completed: run.level_stats.len() as u32,
                            started_at: run.started_at,
                            ended_at: crate::config::save_data::current_timestamp(),
                            wizard_type: config.wizard_type,
                            saved: false,
                            level_stats: run.level_stats.clone(),
                            modifiers: roguelite_modifiers.as_ref().map(|m| m.as_ref().clone()),
                            seed: game_seed.as_ref().map(|s| s.0),
                        };
                        crate::config::save_data::save_roguelite_run(
                            &active_save,
                            roguelite_run_data,
                        );
                    }
                    kill_stats.reset();
                    commands.remove_resource::<RogueliteRunState>();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

/// Accumulates level stats into the roguelite run state after each battle.
/// Also saves endless best stats when in Endless mode.
pub(super) fn accumulate_mode_level_stats(
    game_mode: Option<Res<GameMode>>,
    mut roguelite_run: Option<ResMut<RogueliteRunState>>,
    kill_stats: Res<KillStats>,
    current_level: Res<CurrentLevel>,
    game_outcome: Res<GameOutcome>,
    active_save: Res<ActiveSave>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    // Defeat = 0% efficiency (king died)
    let efficiency = if game_outcome.is_defeat() {
        0.0
    } else {
        let total_defenders = (INITIAL_DEFENDER_COUNT + INITIAL_ARCHER_DEFENDER_COUNT) as f32;
        1.0 - (kill_stats.defenders_killed as f32 / total_defenders)
    };

    let level_stats = LevelRunStats {
        level: current_level.0,
        efficiency,
        attackers_killed: kill_stats.attackers_killed,
        undead_killed: kill_stats.undead_killed,
        defenders_lost: kill_stats.defenders_killed,
        elapsed_time: kill_stats.elapsed_time,
    };

    match game_mode.as_deref() {
        Some(&GameMode::Roguelite) => {
            if let Some(ref mut run) = roguelite_run {
                run.level_stats.push(level_stats);
            }
        }
        Some(&GameMode::Endless) => {
            // Save best stats for this level (only on victory, not during time travel)
            if *game_outcome == GameOutcome::Victory && time_travel.is_none() {
                crate::config::save_data::update_endless_best_stats(&active_save, &level_stats);
            }
        }
        None => {}
    }
}
