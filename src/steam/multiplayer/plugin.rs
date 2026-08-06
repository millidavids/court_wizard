//! Steam-multiplayer plugin: registration only. System bodies live in their
//! sibling files. Only loaded inside `SteamPlugin::Ok` arm — the `Client`
//! resource is guaranteed present when this plugin builds.

use bevy::prelude::*;
use bevy_steamworks::SteamworksEvent;

use super::exit_cleanup::leave_steam_lobby_on_exit;
use super::invite_accept::{
    process_game_lobby_join_requested, process_game_rich_presence_join_requested,
};
use super::join_requests::{
    PendingSteamJoin, parse_launch_command_at_startup, route_pending_steam_join,
};
use super::lobby_members::{process_lobby_chat_updates, sync_coop_peer_name};
use super::lobby_state::SteamLobbyState;
use super::lobby_systems::{
    init_relay_network_access, init_steam_lobby_bridge, process_create_lobby_result,
    process_join_lobby_result,
};
use super::run_conditions::{
    lobby_bridge_ready, steam_guest_dialling, steam_listen_socket_open,
    steam_lobby_traffic_possible, steam_transport_active,
};
use super::socket_bridge::steam_transport_bridge_system;
use super::sockets::{
    SteamP2pSocket, drive_steam_listen_socket, poll_steam_guest_connection_state,
};

pub(crate) struct SteamMultiplayerPlugin;

impl Plugin for SteamMultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SteamLobbyState>()
            .init_resource::<SteamP2pSocket>()
            // SteamLobbyBridge has to be built lazily inside a Startup system
            // because constructing it requires `Res<Client>`.
            .add_systems(
                Startup,
                (init_steam_lobby_bridge, init_relay_network_access),
            )
            .add_systems(Startup, parse_launch_command_at_startup)
            // All of these are gated on DATA, never on `AppState`: the host opens
            // its listen socket from `process_lobby_chat_updates` while sitting in
            // `AppState::Loading`, and an invite can arrive mid-match. A state gate
            // here would reintroduce the stalls this module exists to prevent.
            .add_systems(
                Update,
                (
                    // Steam `FnOnce` call results. Still gated on "traffic possible"
                    // rather than "lobby non-Idle", because a `create_lobby` result
                    // can land after the player cancelled and MUST be drained and
                    // discarded or Steam leaves a ghost lobby advertised.
                    (process_create_lobby_result, process_join_lobby_result)
                        .run_if(steam_lobby_traffic_possible),
                    process_lobby_chat_updates.run_if(lobby_bridge_ready),
                    // Only poll the Steam callback bus when there's actually a
                    // callback waiting — these would otherwise drain an empty
                    // reader every frame in every state.
                    process_game_lobby_join_requested.run_if(on_message::<SteamworksEvent>),
                    process_game_rich_presence_join_requested.run_if(on_message::<SteamworksEvent>),
                    route_pending_steam_join.run_if(resource_exists::<PendingSteamJoin>),
                    drive_steam_listen_socket.run_if(steam_listen_socket_open),
                    poll_steam_guest_connection_state.run_if(steam_guest_dialling),
                    steam_transport_bridge_system.run_if(steam_transport_active),
                    // Only re-read the Steam persona name when the lobby state
                    // changes — not every frame across the FFI boundary.
                    sync_coop_peer_name.run_if(resource_changed::<SteamLobbyState>),
                ),
            )
            // Leave the lobby + clear rich presence before `force_exit_after_save`
            // kills the process (which skips every Drop impl, so nothing else gets
            // to tell Steam we're gone).
            .add_systems(
                Last,
                leave_steam_lobby_on_exit.in_set(crate::config::PreExitCleanupSet),
            );
    }
}
