//! Tell Steam we're leaving before the process is killed.
//!
//! `force_exit_after_save` calls `std::process::exit(0)` on `AppExit`, which skips
//! every `Drop` impl — including `NetConnection`/`ListenSocket` — and never
//! reaches `SteamAPI_Shutdown`. So quitting the game used to leave the lobby
//! advertised and rich presence still publishing `+connect_lobby <dead id>`.
//!
//! That is a "stuck on Connecting" generator no amount of in-session state
//! resetting can fix: a friend clicking Join Game from their friend list is handed
//! a lobby whose host is no longer listening, and the guest dials a socket nobody
//! will ever accept.
//!
//! Clearing rich presence is the load-bearing half — it is what stops the stale
//! invite from being offered at all.

use bevy::prelude::*;
use bevy_steamworks::Client;

use super::lobby_state::{SteamLobbyState, shutdown_steam_session};
use super::sockets::SteamP2pSocket;

/// `Last`, inside `PreExitCleanupSet`: leave the lobby and clear rich presence.
///
/// Runs only when an `AppExit` is actually pending, so this is a no-op on every
/// normal frame.
pub(super) fn leave_steam_lobby_on_exit(
    mut exit_events: MessageReader<AppExit>,
    client: Option<Res<Client>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut socket: Option<ResMut<SteamP2pSocket>>,
) {
    if exit_events.read().count() == 0 {
        return;
    }
    let was_active = lobby_state
        .as_deref()
        .is_some_and(|s| !matches!(*s, SteamLobbyState::Idle));

    shutdown_steam_session(
        client.as_deref(),
        lobby_state.as_deref_mut(),
        socket.as_deref_mut(),
    );

    // `shutdown_steam_session` only clears rich presence when it had a lobby id to
    // leave. Belt-and-braces for the states it can't clean up (notably `Creating`,
    // which has no id yet but may already have published a connect string), clear
    // it unconditionally here — we are exiting, so there is nothing left to
    // advertise either way.
    if let Some(client) = client.as_deref() {
        client.friends().clear_rich_presence();
    }

    if was_active {
        info!("[Steam MP] Left the Steam lobby and cleared rich presence on exit");
    }
}
