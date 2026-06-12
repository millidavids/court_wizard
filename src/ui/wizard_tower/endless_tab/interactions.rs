use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::GameMode;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{CurrentLevel, KillStats, TimeTravelState};
use crate::state::AppState;

use super::super::components::*;
use super::super::layout::RightPanelView;
use super::actions::EndlessAction;

// ---------------------------------------------------------------------------
// Endless action handler
// ---------------------------------------------------------------------------

/// Handles endless tab button actions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_endless_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&EndlessAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut current_level: ResMut<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    active_save: Res<crate::config::ActiveSave>,
    selected_tt_level: Option<Res<SelectedTimeTravelLevel>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut right_panel_view: ResMut<RightPanelView>,
    // Co-op: when a guest is connected, starting endless brings them along.
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
    lobby: Option<Res<super::super::multiplayer_tab::state::MultiplayerLobby>>,
    #[cfg(debug_assertions)] mut level_display: Query<&mut Text, With<LevelDisplay>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = action_query.get(event.button) {
            match action {
                EndlessAction::ContinuePlay => {
                    // Co-op: if connected as host with a guest who picked a wizard,
                    // launch this endless game as co-op — `start_coop_host` inserts
                    // the pending session, seeds terrain, and pulls the guest in via
                    // `StartGame`. (Roguelite uses the same helpers.)
                    if let Some(guest_wizard) =
                        super::super::multiplayer_tab::state::connected_coop_guest_wizard(
                            &connection,
                            lobby.as_deref(),
                        )
                    {
                        crate::game::multiplayer::coop::start_coop_host(
                            &mut commands,
                            &mut connection,
                            &mut config,
                            guest_wizard,
                            crate::networking::session::SessionMode::CoopEndless,
                            false,
                        );
                    }
                    commands.insert_resource(GameMode::Endless);
                    channel_change.write(ChannelChangeMessage);
                    kill_stats.reset();
                    next_app_state.set(AppState::Loading);
                }
                EndlessAction::SwitchWizardType => {
                    *right_panel_view = RightPanelView::WizardSelect;
                }
                EndlessAction::StartTimeTravel => {
                    if let Some(ref sel) = selected_tt_level
                        && let Some(level) = sel.0
                    {
                        // Store the real level so it can be restored after time travel.
                        // Don't modify config.current_level — that tracks actual progression.
                        commands.insert_resource(TimeTravelState {
                            real_level: config.current_level,
                        });
                        commands.insert_resource(GameMode::Endless);
                        // Load the terrain snapshot for the target level so the
                        // battlefield looks like it did when that level was first played.
                        crate::config::save_data::load_level_terrain_into_config(
                            &active_save,
                            level,
                            &mut config,
                        );
                        current_level.0 = level;
                        channel_change.write(ChannelChangeMessage);
                        kill_stats.reset();
                        next_app_state.set(AppState::Loading);
                    }
                }
                #[cfg(debug_assertions)]
                EndlessAction::DebugIncreaseLevel => {
                    config.current_level += 1;
                    if config.current_level > config.highest_level_achieved {
                        config.highest_level_achieved = config.current_level;
                    }
                    current_level.0 = config.current_level;
                    for mut text in &mut level_display {
                        text.0 = format!("Level {}", config.current_level);
                    }
                }
                #[cfg(debug_assertions)]
                EndlessAction::DebugDecreaseLevel => {
                    if config.current_level > 1 {
                        config.current_level -= 1;
                        current_level.0 = config.current_level;
                        for mut text in &mut level_display {
                            text.0 = format!("Level {}", config.current_level);
                        }
                    }
                }
            }
        }
    }
}
