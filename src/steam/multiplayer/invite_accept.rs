//! Accepting an inbound invite: the two `SteamworksEvent` callbacks that record a
//! join intent, and the shared entry point that dispatches `join_lobby`.
//!
//! Both callbacks only record a `PendingSteamJoin`; the routing pipeline
//! (`abandon_run_for_steam_invite` + `route_pending_steam_join`) decides when it
//! is safe to act on it, so an invite works from any game state.

use bevy::prelude::*;
use bevy_steamworks::{CallbackResult, Client, LobbyId, SteamworksEvent};

use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};

use super::lobby_state::{SteamLobbyBridge, SteamLobbyState};

/// Friend clicked "Join Game" on a lobby invite (or accepted an invite overlay)
/// while our game is already running. Steam delivers the lobby ID directly.
pub(super) fn process_game_lobby_join_requested(
    mut events: MessageReader<SteamworksEvent>,
    mut commands: Commands,
) {
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
        commands.insert_resource(super::join_requests::PendingSteamJoin {
            lobby_id: req.lobby_steam_id,
        });
    }
}

/// Friend right-clicked our name and hit "Join Game" from their friend list
/// while our game is running. Steam delivers the `connect` string we set via
/// rich presence (`+connect_lobby <id>`); we parse the lobby ID back out with the
/// same helper used for cold-start launches, then record the intent.
pub(super) fn process_game_rich_presence_join_requested(
    mut events: MessageReader<SteamworksEvent>,
    mut commands: Commands,
) {
    for evt in events.read() {
        let SteamworksEvent::CallbackResult(CallbackResult::GameRichPresenceJoinRequested(req)) =
            evt
        else {
            continue;
        };
        let Some(lobby_id) = super::join_requests::parse_connect_lobby(&req.connect) else {
            warn!(
                "[Steam MP] Could not parse rich-presence connect string '{}' (expected '+connect_lobby <id>')",
                req.connect
            );
            continue;
        };
        info!(
            "[Steam MP] GameRichPresenceJoinRequested: lobby={} (from friend={})",
            lobby_id.raw(),
            req.friend_steam_id.raw()
        );
        commands.insert_resource(super::join_requests::PendingSteamJoin { lobby_id });
    }
}

/// Shared guest-side entry point: set role, mode, and state, then dispatch
/// `join_lobby` whose result will flow through `process_join_lobby_result`.
///
/// Returns `true` if `join_lobby` was actually dispatched. Callers MUST surface a
/// `false` to the player — silently dropping the intent leaves the lobby UI on a
/// permanent "Connecting via Steam relay…" with nothing in flight.
pub(super) fn accept_incoming_join(
    client: &Client,
    bridge: &SteamLobbyBridge,
    lobby_state: &mut SteamLobbyState,
    connection: &mut NetworkConnection,
    lobby_id: LobbyId,
) -> bool {
    // Refuse if we already have any session/lobby in flight. Includes Creating /
    // Hosting so a stray invite can't make us join while we're hosting our own
    // lobby (which would orphan it on Steam). Callers that intend to REPLACE an
    // existing lobby must reset to baseline (→ Idle) first.
    if connection.state == ConnectionState::Connected
        || matches!(
            lobby_state,
            SteamLobbyState::Creating
                | SteamLobbyState::Hosting { .. }
                | SteamLobbyState::Joined { .. }
                | SteamLobbyState::AwaitingJoin { .. }
        )
    {
        warn!(
            "[Steam MP] Ignoring join request — session already in flight (state={:?}, lobby={lobby_state:?})",
            connection.state
        );
        return false;
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
    true
}
