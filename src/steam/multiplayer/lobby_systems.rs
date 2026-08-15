//! Startup wiring plus the systems that drain the two `FnOnce` call-result
//! channels (`create_lobby` / `join_lobby`) and advance the lobby state machine.
//!
//! Membership changes live in `lobby_members.rs`; inbound invite handling lives
//! in `invite_accept.rs`.
//!
//! All UI-state mutation flows through `NetworkConnection` (error / state),
//! which the existing `sync_lobby_with_connection` system in the multiplayer
//! tab translates into `LobbyPhase` changes. That keeps the UI module's
//! internals encapsulated.

use bevy::prelude::*;
use bevy_steamworks::{Client, LobbyId};

use crate::networking::protocol::PROTOCOL_VERSION;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection};

use super::constants::{LOBBY_KEY_PROTOCOL_VERSION, RICH_PRESENCE_CONNECT_KEY};
use super::lobby_state::{SteamLobbyBridge, SteamLobbyState, format_steam_error};
use super::sockets::{SteamP2pSocket, start_connecting, tear_down_socket};

/// Abandon one specific lobby, identified by id rather than by whatever
/// `SteamLobbyState` currently holds.
///
/// The join-result drain is a loop, so state-derived teardown is unsafe here: the
/// first iteration would set `Idle` and every later one would find no id to leave.
fn leave_specific_lobby(
    client: &Client,
    lobby_id: LobbyId,
    lobby_state: &mut SteamLobbyState,
    socket: Option<&mut SteamP2pSocket>,
) {
    client.matchmaking().leave_lobby(lobby_id);
    client.friends().clear_rich_presence();
    if let Some(socket) = socket {
        tear_down_socket(socket);
    }
    *lobby_state = SteamLobbyState::Idle;
}

/// Startup system: construct the bridge (channels + persistent
/// LobbyChatUpdate callback handle) and insert it as a resource.
pub(super) fn init_steam_lobby_bridge(mut commands: Commands, client: Res<Client>) {
    commands.insert_resource(SteamLobbyBridge::new(&client));
}

/// Startup system: kick off SDR relay-network initialization. Documented
/// best-practice when anticipating P2P connections (avoids multi-second
/// startup latency on the first `connect_p2p` call).
pub(super) fn init_relay_network_access(client: Res<Client>) {
    client.networking_utils().init_relay_network_access();
    info!("Steam relay network access initialized");
}

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
                // from their friend list. Steam hands this string back verbatim — on the
                // command line for a cold launch (`GetLaunchCommandLine`), or via the
                // `GameRichPresenceJoinRequested` callback if the game is already running.
                // So it must BE a launch command line: `+connect_lobby <id>`, parsed by
                // the same `parse_connect_lobby` used for cold-start joins.
                client.friends().set_rich_presence(
                    RICH_PRESENCE_CONNECT_KEY,
                    Some(&format!("+connect_lobby {}", lobby_id.raw())),
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
                    // Leave THIS result's lobby by id, then do the cleanup this path
                    // used to skip: rich presence (otherwise we keep advertising
                    // `+connect_lobby <id>` for a lobby we just left, and a friend
                    // clicking Join is handed a dead one) and the P2P socket.
                    //
                    // Deliberately not routed through `shutdown_steam_session`: that
                    // derives the id from `*lobby_state` and then sets it to `Idle`, but
                    // this is a drain loop — a second mismatched result would find
                    // `Idle`, match the "no id" arm, and leave its lobby joined forever.
                    leave_specific_lobby(client, lobby_id, lobby_state, socket.as_deref_mut());
                    connection.error = Some(reason);
                    connection.state = ConnectionState::Failed;
                    continue;
                }

                // `SteamP2pSocket` is `init_resource`, so this is only `None` in a
                // build without the Steam plugin. Bail before advancing any state:
                // the old code moved to `Joined` + `Connecting` regardless, which
                // meant `start_connecting` never ran and never got another chance —
                // `poll_steam_guest_connection_state` then returned early on the
                // absent `socket.connection` every frame, forever.
                let Some(socket) = socket.as_deref_mut() else {
                    warn!("[Steam MP] Joined lobby but no P2P socket resource — cannot dial host");
                    // No socket to tear down here by definition, but rich presence
                    // still has to be cleared. Same id-not-state rule as above.
                    leave_specific_lobby(client, lobby_id, lobby_state, None);
                    connection.error = Some(
                        "Steam networking is unavailable. Try restarting the game.".to_string(),
                    );
                    connection.state = ConnectionState::Failed;
                    continue;
                };

                info!(
                    "[Steam MP] Joined lobby {} (host={}), initiating P2P",
                    lobby_id.raw(),
                    host.raw()
                );
                *lobby_state = SteamLobbyState::Joined {
                    lobby_id,
                    peer: host,
                };
                start_connecting(client, socket, &mut connection, host);
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

/// Drop a `LobbyId` whose `create_lobby` result arrived after the player
/// already cancelled (or switched to iroh). Steam created the lobby on our
/// behalf, so we have to leave it explicitly or it sits there advertising a
/// ghost session. Only touches rich presence if we created the lobby — the
/// lobby_id we got here is proof of that.
fn discard_stale_lobby(client: &Client, lobby_id: LobbyId) {
    client.matchmaking().leave_lobby(lobby_id);
    client.friends().clear_rich_presence();
}
