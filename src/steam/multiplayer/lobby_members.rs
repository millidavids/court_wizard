//! Lobby membership: who is in the Steam lobby, and what to do when that changes.
//!
//! Drains the persistent `LobbyChatUpdate` subscription (registered in
//! `SteamLobbyBridge::new`) and mirrors the connected friend's persona name for
//! the UI. Peer-enter is what opens the host's listen socket; peer-leave is the
//! earliest signal we get that a session is over.

use bevy::prelude::*;
use bevy_steamworks::{ChatMemberStateChange, Client};

use crate::networking::resources::{ConnectionState, NetworkConnection};

use super::lobby_state::{SteamLobbyBridge, SteamLobbyState, leave_steam_lobby};
use super::sockets::{SteamP2pSocket, start_listening, tear_down_socket};

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
                    // Don't advance to `Joined` unless we can actually open the
                    // listen socket. `Entered` fires once and is never replayed, so
                    // advancing without listening left the host waiting on
                    // "Waiting for your friend to accept…" forever while the guest
                    // dialled a socket nobody was listening on.
                    let Some(socket) = socket.as_deref_mut() else {
                        warn!(
                            "[Steam MP] Guest entered lobby {} but no P2P socket resource — cannot listen",
                            lobby_id.raw()
                        );
                        connection.error = Some(
                            "Steam networking is unavailable. Try restarting the game.".to_string(),
                        );
                        connection.state = ConnectionState::Failed;
                        continue;
                    };
                    info!(
                        "[Steam MP] Guest {} entered lobby {}, opening listen socket",
                        peer.raw(),
                        lobby_id.raw()
                    );
                    *lobby_state = SteamLobbyState::Joined { lobby_id, peer };
                    start_listening(client, socket, &mut connection, peer);
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

/// Mirrors the connected Steam friend's persona name into `CoopPeerInfo` so the
/// wizard-tower header and co-op save tagging can show "<name> connected".
/// Cleared to `None` whenever we're not in a lobby (then the UI falls back to a
/// generic "MP connected").
pub(super) fn sync_coop_peer_name(
    client: Res<Client>,
    lobby_state: Res<SteamLobbyState>,
    mut peer_info: ResMut<crate::game::multiplayer::coop::CoopPeerInfo>,
) {
    let name = match *lobby_state {
        SteamLobbyState::Joined { peer, .. } => Some(client.friends().get_friend(peer).name()),
        _ => None,
    };
    if peer_info.name != name {
        peer_info.name = name;
    }
}
