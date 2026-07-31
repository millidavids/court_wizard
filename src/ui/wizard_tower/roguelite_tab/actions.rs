use bevy::prelude::*;

use crate::config::save_data::{self, RogueliteRun};
use crate::config::{ActiveSave, ConfigChanged, GameConfig};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{
    ActiveToggles, GameMode, RogueliteModifiers, RogueliteRunState, ToggleModifier,
};
use crate::game::input::messages::MouseClicked;
use crate::state::AppState;

use super::components::{PendingToggles, RogueliteAction};

/// Handles roguelite action button clicks (Start Run, End Run, Continue, Change Wizard).
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_roguelite_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&RogueliteAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut config: ResMut<GameConfig>,
    mut active_save: ResMut<ActiveSave>,
    pending_toggles: Option<Res<PendingToggles>>,
    roguelite_run: Option<Res<RogueliteRunState>>,
    roguelite_modifiers: Option<Res<RogueliteModifiers>>,
    active_toggles: Option<Res<ActiveToggles>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut config_events: MessageWriter<ConfigChanged>,
    // Co-op: when a guest is connected, starting/continuing a run brings them in.
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
    lobby: Option<Res<super::super::multiplayer_tab::state::MultiplayerLobby>>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match action {
            RogueliteAction::StartRun => {
                channel_change.write(ChannelChangeMessage);

                // Insert game mode
                commands.insert_resource(GameMode::Roguelite);

                // Create ActiveToggles from pending
                let toggles = pending_toggles
                    .as_ref()
                    .map(|p| ActiveToggles::new(p.enabled.clone()))
                    .unwrap_or_default();
                let urgent_active = toggles.is_active(ToggleModifier::Urgent);
                commands.insert_resource(toggles);

                save_data::load_or_create_wizard(config.wizard_type, &mut config, &mut active_save);

                // Initialize roguelite run state (normally done by init_roguelite_run
                // on OnEnter(MetaGame), but we're already in MetaGame)
                config.current_level = 1;
                config.saved_walls.clear();
                config.saved_crystals.clear();
                config.saved_flora.clear();
                config.saved_trampling = Default::default();
                config.saved_trees.clear();
                config.saved_ponds.clear();
                config.saved_bushes.clear();
                config.saved_boulders.clear();
                config.efficiency_ratios.clear();
                commands.insert_resource(RogueliteRunState {
                    started_at: save_data::current_timestamp(),
                    level_stats: vec![],
                    used_non_mouse_input: false,
                });

                config_events.write(ConfigChanged);

                // Co-op: bring the connected guest into this roguelite run.
                if let Some(gw) = super::super::multiplayer_tab::state::connected_coop_guest_wizard(
                    &connection,
                    lobby.as_deref(),
                ) {
                    crate::game::multiplayer::coop::start_coop_host(
                        &mut commands,
                        &mut connection,
                        &mut config,
                        gw,
                        crate::networking::session::SessionMode::CoopRoguelite,
                        urgent_active,
                    );
                }

                // Transition to loading
                next_app_state.set(AppState::Loading);
            }
            RogueliteAction::ContinueRun => {
                channel_change.write(ChannelChangeMessage);
                let urgent_active = active_toggles
                    .as_ref()
                    .is_some_and(|t| t.is_active(ToggleModifier::Urgent));
                if let Some(gw) = super::super::multiplayer_tab::state::connected_coop_guest_wizard(
                    &connection,
                    lobby.as_deref(),
                ) {
                    crate::game::multiplayer::coop::start_coop_host(
                        &mut commands,
                        &mut connection,
                        &mut config,
                        gw,
                        crate::networking::session::SessionMode::CoopRoguelite,
                        urgent_active,
                    );
                }
                next_app_state.set(AppState::Loading);
            }
            RogueliteAction::EndRun => {
                // Save run to history
                if let Some(ref run) = roguelite_run {
                    let roguelite_run_data = RogueliteRun {
                        victory: false,
                        levels_completed: run.level_stats.len() as u32,
                        started_at: run.started_at,
                        ended_at: save_data::current_timestamp(),
                        wizard_type: config.wizard_type,
                        saved: false,
                        level_stats: run.level_stats.clone(),
                        modifiers: roguelite_modifiers.as_ref().map(|m| m.as_ref().clone()),
                        seed: config.seed,
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
                    save_data::save_roguelite_run(&active_save, roguelite_run_data);
                }

                // Clear the dormant run from disk
                save_data::clear_current_roguelite_run(&active_save);

                // Remove run resources
                commands.remove_resource::<RogueliteRunState>();
                commands.remove_resource::<RogueliteModifiers>();
                commands.remove_resource::<ActiveToggles>();
                commands.remove_resource::<GameMode>();

                // Panel rebuild is triggered automatically by the
                // resource_removed::<RogueliteRunState> run condition.
            }
            RogueliteAction::ChangeWizardType => {
                // Switch the right panel view to the wizard select grid
                commands.insert_resource(super::super::layout::RightPanelView::WizardSelect);
            }
        }
    }
}
