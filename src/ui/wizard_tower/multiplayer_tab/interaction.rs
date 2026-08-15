//! Button-action dispatcher for the multiplayer tab.
//!
//! Keyboard input lives in `text_input.rs`. Network message pump and game-start
//! handshake live in `lobby_messages.rs`.

use bevy::prelude::*;
use bevy_steamworks::Client;

use crate::game::input::messages::MouseClicked;
use crate::game::multiplayer::session_reset::reset_multiplayer_to_baseline;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::state::AppState;
use crate::steam::multiplayer::{
    SteamLobbyBridge, SteamLobbyState, SteamP2pSocket, request_steam_invite,
};
use crate::ui::wizard_tower::layout::{RightPanelView, WizardTowerTab};

use super::state::{
    CoopHostSelection, JoinCodeInputBox, LobbyPhase, MpTabAction, MultiplayerLobby,
};

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
    steam_client: Option<Res<Client>>,
    steam_bridge: Option<Res<SteamLobbyBridge>>,
    mut steam_lobby: Option<ResMut<SteamLobbyState>>,
    mut steam_socket: Option<ResMut<SteamP2pSocket>>,
    mut host_selection: ResMut<CoopHostSelection>,
    session: Option<Res<MultiplayerSession>>,
) {
    let send = |cmd: TransportCommand| {
        if let Some(ref t) = transport {
            t.send_command(cmd);
        }
    };
    let session_present = session.is_some();

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
                // Debounce. `CreateHost` is serial on the transport runtime, so a
                // double-click queues a second one that `wait_for_disconnect` now
                // converts into a Failed panel — the button would punish the very
                // impatience it invites.
                if matches!(lobby.phase, LobbyPhase::Hosting) {
                    continue;
                }
                // Tear down anything still live first, exactly as `JoinGame` does.
                // Without this we fire `CreateHost` into a runtime that may still be
                // winding the previous session down (a co-op host whose guest dropped
                // arrives here with `mode = Steam`, a dead socket and a live Steam
                // lobby), and the command gets eaten by `wait_for_disconnect`.
                reset_multiplayer_to_baseline(
                    "starting a new host",
                    &mut commands,
                    &mut connection,
                    &mut lobby,
                    &mut host_selection,
                    transport.as_deref(),
                    steam_client.as_deref(),
                    steam_lobby.as_deref_mut(),
                    steam_socket.as_deref_mut(),
                    session_present,
                );
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
                // Starting a fresh join, so tear down anything still live first.
                // A co-op guest sitting in the tower between levels still holds
                // the host's Steam lobby and a live `SteamP2pSocket`; dialling a
                // host over iroh on top of that leaves the old session dangling
                // and the player advertised in a lobby they have left.
                //
                // This runs here rather than in `ConfirmJoin` for two reasons:
                // the reset clears `join_code_input`, which `ConfirmJoin` needs,
                // and it resets `lobby.phase`, which would drop the player out of
                // the code-entry screen they are standing on. `ConfirmJoin` is
                // only reachable through this arm, so covering it here is enough.
                reset_multiplayer_to_baseline(
                    "starting a new join",
                    &mut commands,
                    &mut connection,
                    &mut lobby,
                    &mut host_selection,
                    transport.as_deref(),
                    steam_client.as_deref(),
                    steam_lobby.as_deref_mut(),
                    steam_socket.as_deref_mut(),
                    session_present,
                );
                lobby.status_message = None;
                lobby.phase = LobbyPhase::Joining;
                lobby.join_code_input.clear();
                lobby.join_code_focused = false;
            }
            MpTabAction::SteamInvite => {
                if let (Some(client), Some(bridge), Some(lobby_state)) = (
                    steam_client.as_deref(),
                    steam_bridge.as_deref(),
                    steam_lobby.as_deref_mut(),
                ) {
                    // A non-Idle lobby means one of two very different things, and
                    // they need opposite handling.
                    match lobby_state {
                        // Our OWN invite is already in flight. This is the
                        // double-click / repeated-event case: starting a second
                        // `create_lobby` would orphan the first (and
                        // `leave_steam_lobby` can't clean up a `Creating` lobby —
                        // it has no id yet). Bail, but say so: silently ignoring
                        // the click is what made this button feel broken.
                        SteamLobbyState::Creating | SteamLobbyState::Hosting { .. } => {
                            lobby.status_message =
                                Some("Invite already open — check the Steam overlay.".to_string());
                            continue;
                        }
                        // Leftovers from a PREVIOUS session that nothing tore down
                        // (e.g. a co-op host who quit to the main menu). These have
                        // a real lobby id, so the reset can actually leave them —
                        // without this the Idle guard blocked the button forever
                        // and only restarting the game recovered.
                        SteamLobbyState::Joined { .. } | SteamLobbyState::AwaitingJoin { .. } => {
                            reset_multiplayer_to_baseline(
                                "stale steam lobby before new invite",
                                &mut commands,
                                &mut connection,
                                &mut lobby,
                                &mut host_selection,
                                transport.as_deref(),
                                Some(client),
                                Some(lobby_state),
                                steam_socket.as_deref_mut(),
                                session_present,
                            );
                        }
                        SteamLobbyState::Idle => {}
                    }
                    lobby.status_message = None;
                    request_steam_invite(client, lobby_state, bridge);
                    connection.state = ConnectionState::WaitingForSignaling;
                    connection.role = Some(PeerRole::Host);
                    connection.mode = ConnectionMode::Steam;
                    lobby.phase = LobbyPhase::SteamHosting;
                } else {
                    lobby.status_message = Some(
                        "Steam isn't running — use Host Game to share a code instead.".to_string(),
                    );
                }
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
            // Cancel, Retry and Disconnect are the same operation from the lobby's
            // point of view: abandon whatever is in flight and go back to a clean
            // Connect screen. Retry in particular MUST be a full baseline reset —
            // it is the button a player reaches for after a failure, and the old
            // partial reset left `peer_protocol_version` and any `MultiplayerSession`
            // behind, which then blocked the next handshake silently.
            MpTabAction::Cancel | MpTabAction::Retry | MpTabAction::Disconnect => {
                reset_multiplayer_to_baseline(
                    "lobby cancel/retry/disconnect",
                    &mut commands,
                    &mut connection,
                    &mut lobby,
                    &mut host_selection,
                    transport.as_deref(),
                    steam_client.as_deref(),
                    steam_lobby.as_deref_mut(),
                    steam_socket.as_deref_mut(),
                    session_present,
                );
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

/// Cancels a still-waiting host "Host Game" attempt when the host navigates away
/// from the multiplayer tabs (Multiplayer / VS) to a game-mode/Study tab.
///
/// Only fires while the host is still WAITING for a guest (`state != Connected`):
/// once a guest is connected the host legitimately browses the Endless/Roguelite/
/// VS tabs to pick a co-op mode, and that live session must be preserved. Switching
/// between the two multiplayer tabs (Multiplayer ↔ VS) also never cancels. This
/// guarantees no hanging host endpoint is left bound after abandoning a Host-Game
/// attempt, and the connection string is only regenerated by clicking "Host Game".
#[allow(clippy::too_many_arguments)]
pub(crate) fn cancel_host_on_tab_leave(
    tab: Res<WizardTowerTab>,
    mut prev_tab: Local<Option<WizardTowerTab>>,
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut host_selection: ResMut<CoopHostSelection>,
    transport: Option<Res<TransportHandle>>,
    steam_client: Option<Res<Client>>,
    mut steam_lobby: Option<ResMut<SteamLobbyState>>,
    mut steam_socket: Option<ResMut<SteamP2pSocket>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let current = *tab;
    let Some(previous) = prev_tab.replace(current) else {
        return;
    };
    if previous == current {
        return;
    }
    let is_mp_tab =
        |t: WizardTowerTab| matches!(t, WizardTowerTab::Multiplayer | WizardTowerTab::Vs);
    if is_mp_tab(previous)
        && !is_mp_tab(current)
        && connection.role == Some(PeerRole::Host)
        && connection.state != ConnectionState::Connected
    {
        reset_multiplayer_to_baseline(
            "host left the multiplayer tabs while waiting",
            &mut commands,
            &mut connection,
            &mut lobby,
            &mut host_selection,
            transport.as_deref(),
            steam_client.as_deref(),
            steam_lobby.as_deref_mut(),
            steam_socket.as_deref_mut(),
            session.is_some(),
        );
    }
}
