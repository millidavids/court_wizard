use bevy::prelude::*;

use crate::config::{ActiveSave, GameConfig};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{GameMode, RogueliteRunState, is_roguelite_mode};
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{GameOutcome, KillStats};
use crate::game::time_travel::TimeTravelState;
use crate::state::AppState;

use crate::networking::resources::NetworkConnection;
use crate::ui::wizard_tower::WizardTowerTab;

use super::super::components::*;
use super::super::saves::save_dormant_roguelite_run;

/// Routes the score screen's "Wizard Tower" transition. Solo players land on
/// the Study tab so spell/talent upgrades are immediately at hand between
/// levels. A co-op host with a guest still attached lands on the active mode
/// tab instead: its Continue button is the only flow that re-invites the guest
/// (`start_coop_host`), and the Study panel has no way to start the next level.
///
/// The pre-inserted tab survives `setup_wizard_tower_layout` only because
/// `RogueliteRunState` is still memory-resident at score-screen time, which
/// suppresses that function's dormant-run restore branch (it would otherwise
/// force the Roguelite tab).
fn enter_wizard_tower(
    commands: &mut Commands,
    next_app_state: &mut NextState<AppState>,
    connection: &NetworkConnection,
    game_mode: Option<&GameMode>,
) {
    // Same gate as the tower's `compute_guest_pending` and the score screen's button
    // list, so the landing tab, the co-op Continue button it carries, and the buttons
    // that got us here can't disagree.
    let coop_guest_attached = connection.has_connected_guest();
    let tab = if !coop_guest_attached {
        WizardTowerTab::Study
    } else if is_roguelite_mode(game_mode) {
        WizardTowerTab::Roguelite
    } else {
        WizardTowerTab::Endless
    };
    commands.insert_resource(tab);
    next_app_state.set(AppState::MetaGame);
}

use crate::game::multiplayer::coop::notify_guest_run_ended;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&GameOverButtonAction>,
    game_outcome: Res<GameOutcome>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    config: Res<GameConfig>,
    time_travel: Option<Res<TimeTravelState>>,
    // Grouped into one tuple param to stay under Bevy's 16-param system limit;
    // the four are always consumed together by `save_dormant_roguelite_run`.
    (roguelite_run, roguelite_modifiers, active_toggles, game_seed): (
        Option<Res<RogueliteRunState>>,
        Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
        Option<Res<crate::game::game_mode::components::ActiveToggles>>,
        Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
    ),
    game_mode: Option<Res<GameMode>>,
    mut connection: ResMut<NetworkConnection>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            match action {
                GameOverButtonAction::NextLevel => {
                    kill_stats.reset();
                    // Persist roguelite progress so the run can be resumed if
                    // the player quits during the next level. No-op in Endless.
                    save_dormant_roguelite_run(
                        &active_save,
                        &roguelite_run,
                        &config,
                        &roguelite_modifiers,
                        &active_toggles,
                        &game_seed,
                    );
                    next_app_state.set(AppState::Loading);
                }
                GameOverButtonAction::PlayAgain => {
                    if time_travel.is_some() {
                        if game_outcome.is_defeat() {
                            // Retry the time-traveled level (TimeTravelState persists)
                            kill_stats.reset();
                            next_app_state.set(AppState::Loading);
                        } else {
                            // Time travel victory: back to the tower. The replay is
                            // torn down by `finish_time_travel` on entering MetaGame,
                            // late enough that OnExit(InGame) teardown still sees the
                            // marker and skips writing replay state to the save.
                            enter_wizard_tower(
                                &mut commands,
                                &mut next_app_state,
                                &connection,
                                game_mode.as_deref(),
                            );
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
                        enter_wizard_tower(
                            &mut commands,
                            &mut next_app_state,
                            &connection,
                            game_mode.as_deref(),
                        );
                    }
                }
                GameOverButtonAction::ReturnToTower => {
                    kill_stats.reset();
                    // Time travel teardown runs in `finish_time_travel` on entering
                    // MetaGame / MainMenu, which covers this button, the score
                    // screen's Main Menu button, and both abandon paths.
                    // Save roguelite run to disk for resume
                    save_dormant_roguelite_run(
                        &active_save,
                        &roguelite_run,
                        &config,
                        &roguelite_modifiers,
                        &active_toggles,
                        &game_seed,
                    );
                    enter_wizard_tower(
                        &mut commands,
                        &mut next_app_state,
                        &connection,
                        game_mode.as_deref(),
                    );
                }
                GameOverButtonAction::ReturnToMenu => {
                    notify_guest_run_ended(&mut connection, !game_outcome.is_defeat());
                    kill_stats.reset();
                    // Abandon roguelite run if exiting to menu from score screen
                    if is_roguelite_mode(game_mode.as_deref()) {
                        crate::config::save_data::clear_current_roguelite_run(&active_save);
                    }
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
                GameOverButtonAction::EndRun => {
                    notify_guest_run_ended(&mut connection, !game_outcome.is_defeat());
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
