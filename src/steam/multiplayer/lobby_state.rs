//! Lobby state machine + crossbeam channels bridging Steam's callback model
//! (FnOnce call results + `register_callback` for `LobbyChatUpdate`) onto the
//! Bevy main thread.

use bevy::prelude::*;
use bevy_steamworks::{Client, LobbyChatUpdate, LobbyId, LobbyType, SResult, SteamError, SteamId};
use crossbeam_channel::{Receiver, Sender, unbounded};

/// Where we are in the Steam-side lifecycle. Independent of `NetworkConnection.state`
/// — that one tracks the data-channel; this one tracks the lobby + signalling.
#[derive(Resource, Default, Debug, Clone)]
pub(crate) enum SteamLobbyState {
    /// No lobby active.
    #[default]
    Idle,

    /// `create_lobby` in flight, waiting for the FnOnce result.
    Creating,

    /// Host owns this lobby, awaiting friend acceptance + LobbyChatUpdate entry.
    Hosting { lobby_id: LobbyId },

    /// Guest sent `join_lobby`, awaiting the FnOnce result.
    AwaitingJoin { lobby_id: LobbyId },

    /// Both peers are in the lobby; we know the other peer's `SteamId`.
    /// `peer` isn't currently read by any system (the socket's `expected_peer`
    /// drives filtering) but stays here so future code can render "Connected
    /// to <friend name>" without round-tripping through Steam again.
    Joined { lobby_id: LobbyId, peer: SteamId },
}

/// Holds the crossbeam senders that Steam-callback closures capture, plus the
/// persistent `CallbackHandle` for `LobbyChatUpdate` (dropping it would
/// unsubscribe).
#[derive(Resource)]
pub(crate) struct SteamLobbyBridge {
    pub(super) create_lobby_tx: Sender<SResult<LobbyId>>,
    pub(super) create_lobby_rx: Receiver<SResult<LobbyId>>,
    pub(super) join_lobby_tx: Sender<Result<LobbyId, ()>>,
    pub(super) join_lobby_rx: Receiver<Result<LobbyId, ()>>,
    pub(super) chat_update_rx: Receiver<LobbyChatUpdate>,
    // Holding this keeps the persistent `LobbyChatUpdate` subscription alive.
    // Dropping it unsubscribes immediately.
    _chat_update_handle: bevy_steamworks::CallbackHandle,
}

impl SteamLobbyBridge {
    /// Build the bridge: allocate three channel pairs and register the
    /// persistent `LobbyChatUpdate` callback.
    pub(crate) fn new(client: &Client) -> Self {
        let (create_lobby_tx, create_lobby_rx) = unbounded();
        let (join_lobby_tx, join_lobby_rx) = unbounded();
        let (chat_update_tx, chat_update_rx) = unbounded();

        // bevy_steamworks::Client derefs to steamworks::Client.
        let chat_update_handle = client.register_callback(move |update: LobbyChatUpdate| {
            let _ = chat_update_tx.send(update);
        });

        Self {
            create_lobby_tx,
            create_lobby_rx,
            join_lobby_tx,
            join_lobby_rx,
            chat_update_rx,
            _chat_update_handle: chat_update_handle,
        }
    }
}

/// Begin hosting: kick off `create_lobby` with a `FnOnce` that forwards the
/// result over the bridge's `create_lobby_tx`. All post-create work
/// (overlay + rich presence + lobby data + state transition) happens in
/// `process_create_lobby_result` on the Bevy main thread.
pub(crate) fn request_steam_invite(
    client: &Client,
    lobby_state: &mut SteamLobbyState,
    bridge: &SteamLobbyBridge,
) {
    let tx = bridge.create_lobby_tx.clone();
    client
        .matchmaking()
        .create_lobby(LobbyType::Private, 2, move |result: SResult<LobbyId>| {
            let _ = tx.send(result);
        });
    *lobby_state = SteamLobbyState::Creating;
}

/// Tear down the lobby (if any). Idempotent: no-op on `Idle`.
pub(crate) fn leave_steam_lobby(client: &Client, lobby_state: &mut SteamLobbyState) {
    let lobby_id = match lobby_state {
        SteamLobbyState::Hosting { lobby_id }
        | SteamLobbyState::AwaitingJoin { lobby_id }
        | SteamLobbyState::Joined { lobby_id, .. } => Some(*lobby_id),
        SteamLobbyState::Idle | SteamLobbyState::Creating => None,
    };

    // Only touch rich presence if we actually had a lobby — otherwise we'd
    // clobber unrelated rich-presence keys (set by other features today or in
    // the future) on every Cancel of an idle Connect panel.
    if let Some(id) = lobby_id {
        client.matchmaking().leave_lobby(id);
        client.friends().clear_rich_presence();
    }
    *lobby_state = SteamLobbyState::Idle;
}

/// Surface a string-friendly description of a `SteamError` for the lobby UI.
pub(super) fn format_steam_error(err: &SteamError) -> String {
    format!("Steam error: {err:?}")
}

/// Convenience: do both `leave_steam_lobby` + `tear_down_socket` for the
/// in-game disconnect paths and `reset_lobby_on_exit`. Each in-game system
/// holds `Option<...>` for these resources (Steam may not be initialized);
/// this wrapper is a no-op when any are missing.
pub(crate) fn shutdown_steam_session(
    client: Option<&Client>,
    lobby_state: Option<&mut SteamLobbyState>,
    socket: Option<&mut super::sockets::SteamP2pSocket>,
) {
    if let (Some(client), Some(lobby_state)) = (client, lobby_state) {
        leave_steam_lobby(client, lobby_state);
    }
    if let Some(socket) = socket {
        super::sockets::tear_down_socket(socket);
    }
}
