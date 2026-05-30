//! Steam-multiplayer plugin: registration only. System bodies live in their
//! sibling files. Only loaded inside `SteamPlugin::Ok` arm — the `Client`
//! resource is guaranteed present when this plugin builds.

use bevy::prelude::*;
use bevy_steamworks::Client;

use super::join_requests::{consume_pending_join_in_main_menu, parse_launch_command_at_startup};
use super::lobby_state::{SteamLobbyBridge, SteamLobbyState};
use super::lobby_systems::{
    process_create_lobby_result, process_game_lobby_join_requested,
    process_game_rich_presence_join_requested, process_join_lobby_result,
    process_lobby_chat_updates,
};
use super::sockets::{
    SteamP2pSocket, drive_steam_listen_socket, poll_steam_guest_connection_state,
    steam_transport_bridge_system,
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
            .add_systems(
                Update,
                (
                    process_create_lobby_result,
                    process_join_lobby_result,
                    process_lobby_chat_updates,
                    process_game_lobby_join_requested,
                    process_game_rich_presence_join_requested,
                    consume_pending_join_in_main_menu,
                    drive_steam_listen_socket,
                    poll_steam_guest_connection_state,
                    steam_transport_bridge_system,
                ),
            );
    }
}

/// Startup system: construct the bridge (channels + persistent
/// LobbyChatUpdate callback handle) and insert it as a resource.
fn init_steam_lobby_bridge(mut commands: Commands, client: Res<Client>) {
    commands.insert_resource(SteamLobbyBridge::new(&client));
}

/// Startup system: kick off SDR relay-network initialization. Documented
/// best-practice when anticipating P2P connections (avoids multi-second
/// startup latency on the first `connect_p2p` call).
fn init_relay_network_access(client: Res<Client>) {
    client.networking_utils().init_relay_network_access();
    info!("Steam relay network access initialized");
}
