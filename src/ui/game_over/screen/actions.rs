use bevy::prelude::*;

use crate::config::{ActiveSave, GameConfig};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{GameMode, RogueliteRunState, is_roguelite_mode};
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{CurrentLevel, GameOutcome, KillStats, TimeTravelState};
use crate::state::AppState;

use super::super::components::*;
use super::super::saves::{insert_wizard_tower_tab, save_dormant_roguelite_run};

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_button_actions(
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
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
    game_mode: Option<Res<GameMode>>,
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
                            insert_wizard_tower_tab(&mut commands, game_mode.as_deref());
                            next_app_state.set(AppState::MetaGame);
                        }
                    } else if game_outcome.is_defeat() {
                        kill_stats.reset();
                        next_app_state.set(AppState::Loading);
                    } else {
                        // Save roguelite run to disk for resume
                        save_dormant_roguelite_run(
                            &active_save,
                            &roguelite_run,
                            &config,
                            &roguelite_modifiers,
                            &active_toggles,
                            &game_seed,
                        );
                        insert_wizard_tower_tab(&mut commands, game_mode.as_deref());
                        next_app_state.set(AppState::MetaGame);
                    }
                }
                GameOverButtonAction::ReturnToTower => {
                    kill_stats.reset();
                    if let Some(ref tt) = time_travel {
                        current_level.0 = tt.real_level;
                        commands.remove_resource::<TimeTravelState>();
                    }
                    // Save roguelite run to disk for resume
                    save_dormant_roguelite_run(
                        &active_save,
                        &roguelite_run,
                        &config,
                        &roguelite_modifiers,
                        &active_toggles,
                        &game_seed,
                    );
                    insert_wizard_tower_tab(&mut commands, game_mode.as_deref());
                    next_app_state.set(AppState::MetaGame);
                }
                GameOverButtonAction::ReturnToMenu => {
                    kill_stats.reset();
                    if time_travel.is_some() {
                        commands.remove_resource::<TimeTravelState>();
                    }
                    // Abandon roguelite run if exiting to menu from score screen
                    if is_roguelite_mode(game_mode.as_deref()) {
                        crate::config::save_data::clear_current_roguelite_run(&active_save);
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
                            active_toggles: active_toggles
                                .as_ref()
                                .map(|t| t.to_ids())
                                .unwrap_or_default(),
                            accessibility_assists: config.has_accessibility_assists(),
                            // Roguelite co-op tagging lands with multi-level co-op
                            // continuation (WS6); co-op is single-level today.
                            played_coop: false,
                            coop_peer_name: None,
                        };
                        crate::config::save_data::save_roguelite_run(
                            &active_save,
                            roguelite_run_data,
                        );
                    }
                    // Clear the dormant run — it's now in history
                    crate::config::save_data::clear_current_roguelite_run(&active_save);
                    kill_stats.reset();
                    commands.remove_resource::<RogueliteRunState>();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}
