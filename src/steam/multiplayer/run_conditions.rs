//! Run conditions for the Steam multiplayer systems.
//!
//! These are deliberately **data**-gated rather than `AppState`-gated. The Steam
//! lobby and socket systems have to keep ticking in every state: the host opens
//! its listen socket from `process_lobby_chat_updates` while sitting in
//! `AppState::Loading`, and an invite can arrive mid-match. Gating them on a menu
//! state would reintroduce exactly the class of stall this module is fixing.

use bevy::prelude::*;

use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};

use super::lobby_state::{SteamLobbyBridge, SteamLobbyState};
use super::sockets::SteamP2pSocket;

/// The lobby-bridge channels only carry traffic once the bridge exists (it is
/// built in a `Startup` system that needs `Res<Client>`).
pub(super) fn lobby_bridge_ready(bridge: Option<Res<SteamLobbyBridge>>) -> bool {
    bridge.is_some()
}

/// True when Steam is the live transport. Both the byte pump and the guest dial
/// poll are no-ops otherwise, and skipping them keeps `ResMut<NetworkConnection>`
/// out of the schedule on every non-Steam frame.
pub(super) fn steam_transport_active(connection: Res<NetworkConnection>) -> bool {
    connection.mode == ConnectionMode::Steam
}

/// Guest-side dial polling: only while we are actually dialling. Once
/// `Connected` the byte pump owns the connection.
pub(super) fn steam_guest_dialling(connection: Res<NetworkConnection>) -> bool {
    connection.mode == ConnectionMode::Steam
        && connection.role == Some(PeerRole::Guest)
        && connection.state != ConnectionState::Connected
}

/// Host-side accept loop: only while a listen socket is open. Without this the
/// system takes the socket `Mutex` every frame for the whole run.
pub(super) fn steam_listen_socket_open(socket: Option<Res<SteamP2pSocket>>) -> bool {
    socket.is_some_and(|s| s.listener.lock().is_ok_and(|slot| slot.is_some()))
}

/// True whenever any Steam lobby work is in flight OR could still arrive.
///
/// Note this deliberately includes `Idle`+Steam-mode: a `create_lobby` result can
/// land *after* the player cancelled, and that late result has to be drained and
/// discarded or Steam leaves a ghost lobby advertised. Gating purely on "non-Idle"
/// would leak exactly that lobby.
pub(super) fn steam_lobby_traffic_possible(
    lobby_state: Option<Res<SteamLobbyState>>,
    connection: Res<NetworkConnection>,
) -> bool {
    let pending = lobby_state.is_some_and(|s| !matches!(*s, SteamLobbyState::Idle));
    pending || connection.mode == ConnectionMode::Steam
}
