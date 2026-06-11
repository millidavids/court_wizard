//! Multiplayer score screen: button clicks, keyboard escape handler, and
//! incoming network message processing (rematch flow, stats reports).

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;
use crate::state::AppState;

use super::super::components::{
    MpRematchState, MpScoreButtonAction, PendingRematch, RematchStatusText,
};
use super::lifecycle::do_mp_disconnect;

/// Handles score screen button clicks.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_mp_score_buttons(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&MpScoreButtonAction>,
    mut rematch_state: ResMut<MpRematchState>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut status_text: Query<&mut Text, With<RematchStatusText>>,
    transport: Option<Res<crate::networking::transport::TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut commands: Commands,
    mut lobby: ResMut<crate::ui::wizard_tower::MultiplayerLobby>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MpScoreButtonAction::Rematch => {
                    rematch_state.local_ready = true;
                    connection
                        .outgoing_messages
                        .push(NetworkMessage::RematchReady);

                    if let Ok(mut text) = status_text.single_mut() {
                        **text = "Waiting for opponent...".to_string();
                    }

                    if rematch_state.remote_ready {
                        commands.insert_resource(PendingRematch);
                        next_app_state.set(AppState::MainMenu);
                    }
                }
                MpScoreButtonAction::Disconnect => {
                    do_mp_disconnect(
                        &mut connection,
                        transport.as_deref(),
                        steam_client.as_deref(),
                        steam_lobby.as_deref_mut(),
                        steam_socket.as_deref_mut(),
                        &mut lobby,
                        &mut commands,
                        &mut next_app_state,
                    );
                }
            }
        }
    }
}

/// Escape on the multiplayer score screen disconnects and returns to the main
/// menu — the same teardown as the score screen's Disconnect button. Registered
/// under `in_mp_score_screen`, so it only fires on the score screen (and NOT in
/// `ButtonActionSet`: this reads the keyboard, not button clicks).
#[allow(clippy::too_many_arguments)]
pub(crate) fn mp_score_escape_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    transport: Option<Res<crate::networking::transport::TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut commands: Commands,
    mut lobby: ResMut<crate::ui::wizard_tower::MultiplayerLobby>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    do_mp_disconnect(
        &mut connection,
        transport.as_deref(),
        steam_client.as_deref(),
        steam_lobby.as_deref_mut(),
        steam_socket.as_deref_mut(),
        &mut lobby,
        &mut commands,
        &mut next_app_state,
    );
}

/// Processes incoming network messages during the score screen.
pub(crate) fn handle_mp_score_messages(
    mut connection: ResMut<NetworkConnection>,
    mut rematch_state: ResMut<MpRematchState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut status_text: Query<&mut Text, With<RematchStatusText>>,
    mut match_stats: Option<ResMut<super::super::score_stats::MatchStats>>,
    mut commands: Commands,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::RematchReady => {
                rematch_state.remote_ready = true;

                if let Ok(mut text) = status_text.single_mut() {
                    if rematch_state.local_ready {
                        **text = "Starting rematch...".to_string();
                    } else {
                        **text = "Opponent wants a rematch!".to_string();
                    }
                }

                if rematch_state.local_ready {
                    commands.insert_resource(PendingRematch);
                    next_app_state.set(AppState::MainMenu);
                }
            }
            NetworkMessage::GameOver { .. } => {
                // Already handled at the game-over transition; ignore here.
            }
            NetworkMessage::WizardStatsReport {
                spell_damage,
                spell_healing,
            } => {
                // Host receives the guest wizard's spell stats and fills in the
                // enemy column of its scoreboard (reactively updated by
                // `update_mp_stat_values`).
                if let Some(stats) = match_stats.as_mut() {
                    stats.enemy_damage = spell_damage;
                    stats.enemy_healing = spell_healing;
                }
            }
            // Keep connection/lobby-level messages for other handlers; DROP
            // stale in-flight gameplay messages (weather, spell hits, avatar
            // control, etc.) — the match is over, so re-queuing them forever
            // just wastes a drain every frame.
            other @ (NetworkMessage::Ping { .. }
            | NetworkMessage::Pong { .. }
            | NetworkMessage::HandshakeVersion { .. }) => unhandled.push(other),
            _ => {}
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
