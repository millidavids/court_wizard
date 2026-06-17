//! Steam-backed multiplayer: SteamMatchmaking lobby for signalling +
//! SteamNetworkingSockets P2P (Steam Datagram Relay) for the data channel.
//!
//! Lives alongside the existing iroh transport. Both bridges write into the
//! shared `NetworkConnection` resource; `NetworkConnection.mode` selects which
//! one is live for a given session.

mod constants;
mod join_requests;
mod lobby_state;
mod lobby_systems;
mod plugin;
mod sockets;

pub(crate) use join_requests::PendingSteamJoin;
pub(crate) use lobby_state::{
    SteamLobbyBridge, SteamLobbyState, leave_steam_lobby, request_steam_invite,
    shutdown_steam_session,
};
pub(crate) use plugin::SteamMultiplayerPlugin;
pub(crate) use sockets::{SteamP2pSocket, tear_down_socket};
