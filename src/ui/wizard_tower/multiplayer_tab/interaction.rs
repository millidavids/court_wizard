//! Button-action dispatcher for the multiplayer tab.
//!
//! Keyboard input lives in `text_input.rs`. Network message pump and game-start
//! handshake live in `lobby_messages.rs`.

use bevy::prelude::*;

use crate::game::input::messages::MouseClicked;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};
use crate::networking::transport::{TransportCommand, TransportHandle};

use super::state::{JoinCodeInputBox, LobbyPhase, MpTabAction, MultiplayerLobby};

/// Processes `MouseClicked` messages and dispatches `MpTabAction`s.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_mp_tab_actions(
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&MpTabAction>,
    input_box_query: Query<Entity, With<JoinCodeInputBox>>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    transport: Option<Res<TransportHandle>>,
) {
    let send = |cmd: TransportCommand| {
        if let Some(ref t) = transport {
            t.send_command(cmd);
        }
    };

    for event in button_clicked.read() {
        if input_box_query.get(event.button).is_ok() {
            lobby.join_code_focused = !lobby.join_code_focused;
            continue;
        }

        let Ok(action) = action_query.get(event.button) else {
            if lobby.join_code_focused {
                lobby.join_code_focused = false;
            }
            continue;
        };

        match action.clone() {
            MpTabAction::HostGame => {
                send(TransportCommand::CreateHost {
                    use_relay: lobby.use_relay,
                });
                connection.state = ConnectionState::WaitingForSignaling;
                connection.role = Some(PeerRole::Host);
                connection.mode = if lobby.use_relay {
                    ConnectionMode::Online
                } else {
                    ConnectionMode::Lan
                };
                lobby.phase = LobbyPhase::Hosting;
            }
            MpTabAction::JoinGame => {
                lobby.phase = LobbyPhase::Joining;
                lobby.join_code_input.clear();
                lobby.join_code_focused = false;
            }
            MpTabAction::ToggleRelay => {
                lobby.use_relay = !lobby.use_relay;
            }
            MpTabAction::CopyCode => {
                if let Some(code) = &connection.local_code
                    && let Ok(mut clipboard) = arboard::Clipboard::new()
                {
                    let _ = clipboard.set_text(code.clone());
                }
            }
            MpTabAction::PasteFromClipboard => {
                match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    Ok(text) if !text.trim().is_empty() => {
                        lobby.join_code_input = text.trim().to_string();
                        lobby.join_code_focused = false;
                    }
                    _ => {
                        connection.error = Some("Could not read from clipboard".to_string());
                    }
                }
            }
            MpTabAction::ConfirmJoin => {
                if !lobby.join_code_input.is_empty() {
                    send(TransportCommand::ConnectToHost {
                        ticket_code: lobby.join_code_input.clone(),
                    });
                    connection.state = ConnectionState::Connecting;
                    connection.role = Some(PeerRole::Guest);
                    connection.mode = ConnectionMode::Online;
                }
            }
            MpTabAction::Cancel | MpTabAction::Retry | MpTabAction::Disconnect => {
                send(TransportCommand::Disconnect);
                connection.reset();
                lobby.phase = LobbyPhase::Connect;
                lobby.join_code_input.clear();
                lobby.join_code_focused = false;
            }
            MpTabAction::Ready => {
                if let LobbyPhase::WizardSelect {
                    my_wizard: Some(_),
                    my_ready,
                    ..
                } = &mut lobby.phase
                {
                    *my_ready = true;
                    connection.outgoing_messages.push(NetworkMessage::ReadyUp);
                }
            }
            MpTabAction::Unready => {
                if let LobbyPhase::WizardSelect { my_ready, .. } = &mut lobby.phase {
                    *my_ready = false;
                    connection.outgoing_messages.push(NetworkMessage::Unready);
                }
            }
            MpTabAction::SelectWizard(wizard_type) => {
                if let LobbyPhase::WizardSelect { my_ready: true, .. } = &lobby.phase {
                    continue;
                }
                if let LobbyPhase::WizardSelect { my_wizard, .. } = &mut lobby.phase {
                    *my_wizard = Some(wizard_type);
                    connection
                        .outgoing_messages
                        .push(NetworkMessage::WizardSelected(wizard_type));
                }
            }
        }
    }
}
