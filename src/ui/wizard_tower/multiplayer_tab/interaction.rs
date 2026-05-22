//! Button-action dispatcher for the multiplayer tab.
//!
//! Keyboard input lives in `text_input.rs`. Network message pump and game-start
//! handshake live in `lobby_messages.rs`.

use bevy::prelude::*;

use crate::game::input::messages::MouseClicked;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::state::AppState;
use crate::ui::wizard_tower::layout::RightPanelView;

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
    mut commands: Commands,
    mut right_panel_view: ResMut<RightPanelView>,
    mut next_app_state: ResMut<NextState<AppState>>,
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
                lobby.status_message = None;
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
                lobby.status_message = None;
                lobby.phase = LobbyPhase::Joining;
                lobby.join_code_input.clear();
                lobby.join_code_focused = false;
            }
            MpTabAction::ToggleRelay => {
                lobby.use_relay = !lobby.use_relay;
            }
            MpTabAction::CopyCode => {
                lobby.status_message = Some(match connection.local_code.clone() {
                    Some(code) => {
                        let copied = arboard::Clipboard::new()
                            .and_then(|mut cb| cb.set_text(code))
                            .is_ok();
                        if copied {
                            "Code copied — send it to your friend!".to_string()
                        } else {
                            "Couldn't copy — clipboard unavailable.".to_string()
                        }
                    }
                    None => "Code not ready yet — wait a moment.".to_string(),
                });
            }
            MpTabAction::PasteFromClipboard => {
                match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    Ok(text) if !text.trim().is_empty() => {
                        lobby.join_code_input = text.trim().to_string();
                        lobby.join_code_focused = false;
                        lobby.status_message = Some("Code pasted — click Connect.".to_string());
                    }
                    _ => {
                        lobby.status_message =
                            Some("Clipboard is empty — copy the host's code first.".to_string());
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
                lobby.status_message = None;
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
            MpTabAction::SwitchWizard => {
                // Can't switch while Ready.
                if matches!(
                    &lobby.phase,
                    LobbyPhase::WizardSelect { my_ready: true, .. }
                ) {
                    continue;
                }
                // Hand off to the shared wizard-card grid.
                *right_panel_view = RightPanelView::WizardSelect;
            }
            MpTabAction::StartGame => {
                super::lobby_messages::commit_host_start(
                    &lobby,
                    &mut connection,
                    &mut commands,
                    &mut next_app_state,
                );
            }
        }
    }
}
