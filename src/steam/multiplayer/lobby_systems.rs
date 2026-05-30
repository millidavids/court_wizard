//! Update systems that drain the lobby-bridge channels + the
//! `SteamworksEvent` message bus and drive the lobby state machine.
//!
//! All UI-state mutation flows through `NetworkConnection` (error / state),
//! which the existing `sync_lobby_with_connection` system in the multiplayer
//! tab translates into `LobbyPhase` changes. That keeps the UI module's
//! internals encapsulated.

use std::str::FromStr;

use bevy::prelude::*;
use bevy_steamworks::{CallbackResult, ChatMemberStateChange, Client, LobbyId, SteamworksEvent};

use crate::networking::protocol::PROTOCOL_VERSION;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};

use super::constants::{LOBBY_KEY_PROTOCOL_VERSION, RICH_PRESENCE_CONNECT_KEY};
use super::lobby_state::{
    SteamLobbyBridge, SteamLobbyState, format_steam_error, leave_steam_lobby,
};
use super::sockets::{SteamP2pSocket, start_connecting, start_listening, tear_down_socket};

/// Drain `create_lobby` FnOnce results pushed from the Steam dispatcher
/// thread. On success: finish the host-side setup (overlay, lobby data,
/// rich presence) on the Bevy main thread. On failure: surface via
/// `NetworkConnection` so `sync_lobby_with_connection` flips the UI to Failed.
pub(super) fn process_create_lobby_result(
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let (Some(client), Some(bridge), Some(lobby_state)) = (
        client.as_deref(),
        bridge.as_deref(),
        lobby_state.as_deref_mut(),
    ) else {
        return;
    };

    while let Ok(result) = bridge.create_lobby_rx.try_recv() {
        match result {
            Ok(lobby_id) => {
                // If the player already cancelled (state reverted to Idle) or
                // switched transports (mode is no longer Steam), this callback
                // is stale — leave the lobby Steam created on our behalf so it
                // doesn't sit there advertising a ghost session.
                if !matches!(lobby_state, SteamLobbyState::Creating)
                    || connection.mode != ConnectionMode::Steam
                {
                    info!(
                        "[Steam MP] Discarding stale create_lobby result (id={}, state={lobby_state:?}, mode={:?})",
                        lobby_id.raw(),
                        connection.mode
                    );
                    discard_stale_lobby(client, lobby_id);
                    continue;
                }

                info!(
                    "[Steam MP] Lobby created (id={}), opening invite overlay",
                    lobby_id.raw()
                );
                *lobby_state = SteamLobbyState::Hosting { lobby_id };

                // Stamp our protocol version so the guest can fast-reject mismatches.
                let _ = client.matchmaking().set_lobby_data(
                    lobby_id,
                    LOBBY_KEY_PROTOCOL_VERSION,
                    &PROTOCOL_VERSION.to_string(),
                );

                // Set rich-presence "connect" key so friends can right-click → Join Game
                // from their friend list (fires `GameRichPresenceJoinRequested` on accept).
                client.friends().set_rich_presence(
                    RICH_PRESENCE_CONNECT_KEY,
                    Some(&lobby_id.raw().to_string()),
                );

                // Open the Steam friends overlay so the host can pick whom to invite.
                client.friends().activate_invite_dialog(lobby_id);
            }
            Err(err) => {
                // Only surface the failure if we were actually waiting on a
                // create_lobby result — a stale Err that arrived after Cancel
                // shouldn't paint the lobby UI as Failed.
                if !matches!(lobby_state, SteamLobbyState::Creating) {
                    warn!("[Steam MP] Discarding stale create_lobby error: {err:?}");
                    continue;
                }
                let reason = format_steam_error(&err);
                warn!("[Steam MP] create_lobby failed: {reason}");
                *lobby_state = SteamLobbyState::Idle;
                connection.error = Some(reason);
                connection.state = ConnectionState::Failed;
            }
        }
    }
}

/// Drain `join_lobby` FnOnce results. On `Ok`: look up the host's SteamId
/// via `lobby_owner`, verify protocol version compatibility from the lobby
/// metadata, transition to `Joined`, and kick off `connect_p2p`. On `Err`:
/// surface via `NetworkConnection.error` + `Failed` so the UI flips to the
/// Failed panel.
pub(super) fn process_join_lobby_result(
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut socket: Option<ResMut<SteamP2pSocket>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let (Some(client), Some(bridge), Some(lobby_state)) = (
        client.as_deref(),
        bridge.as_deref(),
        lobby_state.as_deref_mut(),
    ) else {
        return;
    };

    while let Ok(result) = bridge.join_lobby_rx.try_recv() {
        match result {
            Ok(lobby_id) => {
                let host = client.matchmaking().lobby_owner(lobby_id);

                // Protocol version pre-check: avoids burning the connect
                // round trip just to discover the in-protocol handshake will
                // also fail.
                let local_version = PROTOCOL_VERSION.to_string();
                let remote_version = client
                    .matchmaking()
                    .lobby_data(lobby_id, LOBBY_KEY_PROTOCOL_VERSION);
                if remote_version.as_deref() != Some(local_version.as_str()) {
                    let reason = format!(
                        "Multiplayer version mismatch: you are on v{local_version}, host is on v{}. Both players must update to the same game version.",
                        remote_version.as_deref().unwrap_or("?")
                    );
                    warn!("[Steam MP] {reason}");
                    client.matchmaking().leave_lobby(lobby_id);
                    *lobby_state = SteamLobbyState::Idle;
                    connection.error = Some(reason);
                    connection.state = ConnectionState::Failed;
                    continue;
                }

                info!(
                    "[Steam MP] Joined lobby {} (host={}), initiating P2P",
                    lobby_id.raw(),
                    host.raw()
                );
                *lobby_state = SteamLobbyState::Joined {
                    lobby_id,
                    peer: host,
                };
                if let Some(socket) = socket.as_deref_mut() {
                    start_connecting(client, socket, &mut connection, host);
                }
                // start_connecting flips state to Failed on error; don't
                // overwrite that with Connecting.
                if connection.state != ConnectionState::Failed {
                    connection.state = ConnectionState::Connecting;
                }
            }
            Err(()) => {
                warn!("[Steam MP] join_lobby failed");
                *lobby_state = SteamLobbyState::Idle;
                connection.error = Some("Could not join Steam lobby.".to_string());
                connection.state = ConnectionState::Failed;
            }
        }
    }
}

/// Drain `LobbyChatUpdate` callbacks. On host: when the invited friend
/// enters, open the listen socket and transition to `Joined`. On either
/// side: when the peer leaves/disconnects, tear down the socket + lobby
/// and signal the existing disconnect-detection flow.
pub(super) fn process_lobby_chat_updates(
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut socket: Option<ResMut<SteamP2pSocket>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let (Some(client), Some(bridge), Some(lobby_state)) = (
        client.as_deref(),
        bridge.as_deref(),
        lobby_state.as_deref_mut(),
    ) else {
        return;
    };

    while let Ok(update) = bridge.chat_update_rx.try_recv() {
        // Always use `user_changed` — `making_change` is mis-mapped to the same
        // SteamID field in steamworks 0.12.2 (`matchmaking.rs:1128`).
        let peer = update.user_changed;
        let change = update.member_state_change;

        // Skip our own enter event (we already created the lobby on host,
        // or we're the guest who just entered — handled by join_lobby
        // callback, not here).
        let local_id = client.user().steam_id();
        if peer == local_id {
            continue;
        }

        match change {
            ChatMemberStateChange::Entered => match lobby_state {
                SteamLobbyState::Hosting { lobby_id } => {
                    let lobby_id = *lobby_id;
                    info!(
                        "[Steam MP] Guest {} entered lobby {}, opening listen socket",
                        peer.raw(),
                        lobby_id.raw()
                    );
                    *lobby_state = SteamLobbyState::Joined { lobby_id, peer };
                    if let Some(socket) = socket.as_deref_mut() {
                        start_listening(client, socket, &mut connection, peer);
                    }
                }
                // Idempotent: if we already transitioned, ignore stale events.
                SteamLobbyState::Joined { .. } | SteamLobbyState::Idle => {}
                // Guest path uses join_lobby result, not LobbyChatUpdate.
                SteamLobbyState::Creating | SteamLobbyState::AwaitingJoin { .. } => {}
            },
            ChatMemberStateChange::Left
            | ChatMemberStateChange::Disconnected
            | ChatMemberStateChange::Kicked
            | ChatMemberStateChange::Banned => {
                if matches!(lobby_state, SteamLobbyState::Joined { .. }) {
                    info!(
                        "[Steam MP] Peer {} left (change={:?}), tearing down",
                        peer.raw(),
                        change
                    );
                    if let Some(socket) = socket.as_deref_mut() {
                        tear_down_socket(socket);
                    }
                    leave_steam_lobby(client, lobby_state);
                    // Set a Steam-specific error so the Failed panel reads
                    // "Your friend left the lobby" instead of generic
                    // "Connection lost".
                    connection.error = Some(match change {
                        ChatMemberStateChange::Left => "Your friend left the lobby.".to_string(),
                        ChatMemberStateChange::Disconnected => {
                            "Your friend disconnected from Steam.".to_string()
                        }
                        ChatMemberStateChange::Kicked => {
                            "Your friend was kicked from the lobby.".to_string()
                        }
                        ChatMemberStateChange::Banned => {
                            "Your friend was banned from the lobby.".to_string()
                        }
                        _ => "Your friend left the lobby.".to_string(),
                    });
                    connection.state = ConnectionState::Disconnected;
                }
            }
        }
    }
}

/// Friend clicked "Join Game" on a lobby invite (or accepted an invite
/// overlay) while our game is already running. Steam delivers the lobby ID
/// directly.
pub(super) fn process_game_lobby_join_requested(
    mut events: MessageReader<SteamworksEvent>,
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let (Some(client), Some(bridge), Some(lobby_state)) = (
        client.as_deref(),
        bridge.as_deref(),
        lobby_state.as_deref_mut(),
    ) else {
        // Still drain the events so they don't pile up.
        events.read().for_each(|_| {});
        return;
    };

    for evt in events.read() {
        let SteamworksEvent::CallbackResult(CallbackResult::GameLobbyJoinRequested(req)) = evt
        else {
            continue;
        };
        info!(
            "[Steam MP] GameLobbyJoinRequested: lobby={} friend={}",
            req.lobby_steam_id.raw(),
            req.friend_steam_id.raw()
        );
        accept_incoming_join(
            client,
            bridge,
            lobby_state,
            &mut connection,
            req.lobby_steam_id,
        );
    }
}

/// Friend right-clicked our name and hit "Join Game" from their friend list
/// while our game is running. Steam delivers the `connect` string we set via
/// rich presence — we parse the lobby ID back out.
pub(super) fn process_game_rich_presence_join_requested(
    mut events: MessageReader<SteamworksEvent>,
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let (Some(client), Some(bridge), Some(lobby_state)) = (
        client.as_deref(),
        bridge.as_deref(),
        lobby_state.as_deref_mut(),
    ) else {
        events.read().for_each(|_| {});
        return;
    };

    for evt in events.read() {
        let SteamworksEvent::CallbackResult(CallbackResult::GameRichPresenceJoinRequested(req)) =
            evt
        else {
            continue;
        };
        let Ok(raw) = u64::from_str(&req.connect) else {
            warn!(
                "[Steam MP] Could not parse rich-presence connect string '{}' as lobby id",
                req.connect
            );
            continue;
        };
        let lobby_id = LobbyId::from_raw(raw);
        info!(
            "[Steam MP] GameRichPresenceJoinRequested: lobby={} (from friend={})",
            lobby_id.raw(),
            req.friend_steam_id.raw()
        );
        accept_incoming_join(client, bridge, lobby_state, &mut connection, lobby_id);
    }
}

/// Shared guest-side entry point: set role, mode, and state, then dispatch
/// `join_lobby` whose result will flow through `process_join_lobby_result`.
pub(super) fn accept_incoming_join(
    client: &Client,
    bridge: &SteamLobbyBridge,
    lobby_state: &mut SteamLobbyState,
    connection: &mut NetworkConnection,
    lobby_id: LobbyId,
) {
    // Refuse if we're already mid-session — Steam should never deliver this
    // mid-game in practice but be defensive.
    if connection.state == ConnectionState::Connected
        || matches!(
            lobby_state,
            SteamLobbyState::Joined { .. } | SteamLobbyState::AwaitingJoin { .. }
        )
    {
        warn!("[Steam MP] Ignoring join request — session already in flight");
        return;
    }

    connection.error = None;
    connection.mode = ConnectionMode::Steam;
    connection.role = Some(PeerRole::Guest);
    connection.state = ConnectionState::WaitingForSignaling;
    *lobby_state = SteamLobbyState::AwaitingJoin { lobby_id };

    let tx = bridge.join_lobby_tx.clone();
    client.matchmaking().join_lobby(lobby_id, move |result| {
        let _ = tx.send(result);
    });
}

/// Drop a `LobbyId` whose `create_lobby` result arrived after the player
/// already cancelled (or switched to iroh). Steam created the lobby on our
/// behalf, so we have to leave it explicitly or it sits there advertising a
/// ghost session. Only touches rich presence if we created the lobby — the
/// lobby_id we got here is proof of that.
pub(super) fn discard_stale_lobby(client: &Client, lobby_id: LobbyId) {
    client.matchmaking().leave_lobby(lobby_id);
    client.friends().clear_rich_presence();
}
