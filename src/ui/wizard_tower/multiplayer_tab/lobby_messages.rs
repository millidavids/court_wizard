//! Lobby message pump: drains `NetworkConnection.incoming_messages`,
//! advances `LobbyPhase`, and triggers `StartGame` when both peers are ready.
//!
//! Ported from the legacy multiplayer screen — the host-authoritative game-start
//! handshake is preserved verbatim.

use bevy::prelude::*;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::AppState;

use super::state::{LobbyPhase, MultiplayerLobby, load_my_unlocked_content};

/// Drains `NetworkConnection.incoming_messages`, sends `PlayerInfo` on first
/// connect, handles `PlayerInfo`/`WizardSelected`/`ReadyUp`/`Unready`/`StartGame`.
pub(crate) fn process_lobby_messages(
    connection: ResMut<NetworkConnection>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let should_send_info = connection.state == ConnectionState::Connected
        && matches!(lobby.phase, LobbyPhase::Handshake);

    let has_messages = !connection.incoming_messages.is_empty();

    if !should_send_info && !has_messages {
        return;
    }

    let mut connection = connection;

    if should_send_info {
        let (wizard_types, spells) = load_my_unlocked_content();
        info!(
            "[MP Lobby] Connected — sending PlayerInfo ({} wizard types, {} spells)",
            wizard_types.len(),
            spells.len()
        );
        connection
            .outgoing_messages
            .push(NetworkMessage::PlayerInfo {
                wizard_types: wizard_types.clone(),
                spells: spells.clone(),
            });
    }

    if has_messages {
        let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
        let mut unhandled = Vec::new();

        for msg in messages {
            match msg {
                NetworkMessage::PlayerInfo {
                    wizard_types: opponent_wt,
                    spells: _,
                } => {
                    info!(
                        "[MP Lobby] Received PlayerInfo ({} wizard types)",
                        opponent_wt.len()
                    );
                    let (my_wt, _) = load_my_unlocked_content();
                    let initial = my_wt
                        .first()
                        .copied()
                        .unwrap_or(crate::config::WizardType::BoringOleMage);

                    lobby.phase = LobbyPhase::WizardSelect {
                        my_wizard_types: my_wt.clone(),
                        opponent_wizard_types: opponent_wt,
                        my_wizard: Some(initial),
                        opponent_wizard: None,
                        my_ready: false,
                        opponent_ready: false,
                    };
                    connection
                        .outgoing_messages
                        .push(NetworkMessage::WizardSelected(initial));
                }
                NetworkMessage::WizardSelected(wt) => {
                    if let LobbyPhase::WizardSelect {
                        opponent_wizard, ..
                    } = &mut lobby.phase
                    {
                        *opponent_wizard = Some(wt);
                    }
                }
                NetworkMessage::ReadyUp => {
                    if let LobbyPhase::WizardSelect { opponent_ready, .. } = &mut lobby.phase {
                        *opponent_ready = true;
                    }
                }
                NetworkMessage::Unready => {
                    if let LobbyPhase::WizardSelect { opponent_ready, .. } = &mut lobby.phase {
                        *opponent_ready = false;
                    }
                }
                NetworkMessage::StartGame => {
                    info!("[MP Lobby] Received StartGame from host");
                    if let LobbyPhase::WizardSelect {
                        my_wizard: Some(my_wiz),
                        opponent_wizard: Some(opp_wiz),
                        ..
                    } = &lobby.phase
                    {
                        let (_, my_spells) = load_my_unlocked_content();
                        let session = MultiplayerSession {
                            role: PeerRole::Guest,
                            host_wizard: *opp_wiz,
                            guest_wizard: *my_wiz,
                            host_spells: Vec::new(),
                            guest_spells: my_spells,
                        };
                        commands.insert_resource(session);
                    }
                    next_app_state.set(AppState::MultiplayerLoading);
                }
                other => unhandled.push(other),
            }
        }

        if !unhandled.is_empty() {
            connection.incoming_messages.extend(unhandled);
        }
    }

    check_both_ready(&lobby, &mut connection, &mut commands, &mut next_app_state);
}

/// If local is host AND both wizards selected AND both ready, send `StartGame`
/// and transition to `MultiplayerLoading`.
fn check_both_ready(
    lobby: &MultiplayerLobby,
    connection: &mut NetworkConnection,
    commands: &mut Commands,
    next_app_state: &mut NextState<AppState>,
) {
    if let LobbyPhase::WizardSelect {
        my_wizard: Some(my_wiz),
        opponent_wizard: Some(opp_wiz),
        my_ready: true,
        opponent_ready: true,
        ..
    } = &lobby.phase
        && connection.role == Some(PeerRole::Host)
    {
        info!(
            "[MP Lobby] Both ready! Host: {:?}, Guest: {:?} — sending StartGame",
            my_wiz, opp_wiz
        );
        let (_, my_spells) = load_my_unlocked_content();
        let session = MultiplayerSession {
            role: PeerRole::Host,
            host_wizard: *my_wiz,
            guest_wizard: *opp_wiz,
            host_spells: my_spells,
            guest_spells: Vec::new(),
        };
        commands.insert_resource(session);
        connection.outgoing_messages.push(NetworkMessage::StartGame);
        next_app_state.set(AppState::MultiplayerLoading);
    }
}
