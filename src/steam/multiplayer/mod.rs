//! Steam-backed multiplayer: SteamMatchmaking lobby for signalling +
//! SteamNetworkingSockets P2P (Steam Datagram Relay) for the data channel.
//!
//! Lives alongside the existing iroh transport. Both bridges write into the
//! shared `NetworkConnection` resource; `NetworkConnection.mode` selects which
//! one is live for a given session.

mod constants;
mod exit_cleanup;
mod invite_accept;
mod join_requests;
mod lobby_members;
mod lobby_state;
mod lobby_systems;
mod plugin;
mod run_conditions;
mod socket_bridge;
mod sockets;

pub(crate) use join_requests::PendingSteamJoin;
// `leave_steam_lobby` / `tear_down_socket` are deliberately NOT re-exported: every
// caller outside this module now goes through `shutdown_steam_session` (via
// `session_reset::reset_multiplayer_to_baseline`), so a future teardown path can't
// quietly leave one half of the Steam session standing.
pub(crate) use lobby_state::{
    SteamLobbyBridge, SteamLobbyState, request_steam_invite, shutdown_steam_session,
};
pub(crate) use plugin::SteamMultiplayerPlugin;
pub(crate) use sockets::SteamP2pSocket;
