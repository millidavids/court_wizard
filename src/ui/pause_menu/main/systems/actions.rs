use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, InGameState, PauseMenuState};

use super::super::components::PauseMenuButtonAction;

/// Handles pause menu button actions.
#[allow(clippy::too_many_arguments)]
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&PauseMenuButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    coop_pause: Option<Res<crate::game::multiplayer::coop_pause::CoopPauseState>>,
    // Exit tells an attached co-op guest the run is over before tearing down.
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                PauseMenuButtonAction::Continue => {
                    // In a co-op sync-pause only the initiator may resume.
                    if !crate::game::multiplayer::coop_pause::local_can_resume(
                        session.as_deref(),
                        coop_pause.as_deref(),
                    ) {
                        continue;
                    }
                    next_in_game_state.set(InGameState::Running);
                }
                PauseMenuButtonAction::Settings => {
                    next_pause_menu_state.set(PauseMenuState::Settings);
                }
                PauseMenuButtonAction::Manual => {
                    next_pause_menu_state.set(PauseMenuState::Manual);
                }
                PauseMenuButtonAction::Compendium => {
                    next_pause_menu_state.set(PauseMenuState::Compendium);
                }
                PauseMenuButtonAction::Exit => {
                    crate::game::shared_systems::abandon_run_to_main_menu(
                        &mut active_save,
                        &mut channel_change,
                        &mut next_app_state,
                        &mut connection,
                    );
                }
            }
        }
    }
}

/// `OnEnter(PauseMenuState::Main)` after `setup`: if the local peer is the
/// NON-initiator of an active co-op sync-pause, relabel the Continue button to
/// "Waiting for other player" and mute it (the resume itself is blocked in
/// `button_action` / the gated `escape_to_running`).
pub fn relabel_continue_for_coop(
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    pause: Option<Res<crate::game::multiplayer::coop_pause::CoopPauseState>>,
    buttons: Query<(&PauseMenuButtonAction, &Children)>,
    front_q: Query<&Children, With<crate::ui::components::ButtonFront>>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
) {
    let Some(session) = session else { return };
    let Some(pause) = pause else { return };
    if !session.coop_pause_synced() || !pause.active {
        return;
    }
    if pause.initiator == Some(session.role) {
        return; // the initiator keeps a normal, working Continue button
    }

    for (action, children) in &buttons {
        if matches!(action, PauseMenuButtonAction::Continue) {
            crate::game::multiplayer::coop_pause::set_waiting_label(children, &front_q, &mut texts);
        }
    }
}
