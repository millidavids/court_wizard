//! Abandon the player's own active run when a Steam invite is accepted mid-game.
//!
//! Accepting a Steam invite records a `PendingSteamJoin` intent (see
//! `crate::steam::multiplayer`). If the player is in a menu, the steam-side
//! `route_pending_steam_join` consumes it directly. If they're in an active run,
//! THIS system tears the run down to the main menu first, leaving the intent in
//! place so `route_pending_steam_join` then routes from the menu into the
//! multiplayer tab and connects.
//!
//! Lives here (not in the steam module) so the single-player and multiplayer
//! teardown helpers (`do_mp_disconnect`, `clear_current_roguelite_run`) stay
//! local to the module that owns them.

use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::networking::resources::NetworkConnection;
use crate::state::AppState;

use super::lifecycle::do_mp_disconnect;

/// Tear down the player's own active single-player or multiplayer run so an
/// accepted Steam invite can route to the multiplayer tab. Register gated on
/// `resource_exists::<PendingSteamJoin>` AND (`InGame` OR `MultiplayerGame`); the
/// body branches on the concrete state. Leaves the `PendingSteamJoin` intact —
/// `route_pending_steam_join` consumes it once we reach the menu.
#[allow(clippy::too_many_arguments)]
pub(crate) fn abandon_run_for_steam_invite(
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut connection: ResMut<NetworkConnection>,
    mut commands: Commands,
    // Single-player abandon (mirrors the pause-menu "Exit").
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    // Multiplayer abandon (the `do_mp_disconnect` param set).
    transport: Option<Res<crate::networking::transport::TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    // `MultiplayerLobby` is init_resource (always present), so a non-Option binding
    // turns any future absence into a loud panic rather than a silent stuck-invite.
    mut lobby: ResMut<crate::ui::wizard_tower::MultiplayerLobby>,
    mut host_selection: ResMut<crate::ui::wizard_tower::CoopHostSelection>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    match app_state.get() {
        // Single-player: discard the in-progress run and return to the main menu,
        // exactly like the pause-menu "Exit" button (shared helper).
        AppState::InGame => {
            crate::game::shared_systems::abandon_run_to_main_menu(
                &mut active_save,
                &mut channel_change,
                &mut next_app_state,
                &mut connection,
            );
        }
        // Multiplayer: end the current session cleanly (leaves the old lobby +
        // socket, clears rich presence, resets the connection) and return to the
        // main menu. Does NOT touch `ActiveSave` — the player's dormant SP run, if
        // any, must survive abandoning an unrelated MP match.
        AppState::MultiplayerGame => {
            do_mp_disconnect(
                &mut connection,
                transport.as_deref(),
                steam_client.as_deref(),
                steam_lobby.as_deref_mut(),
                steam_socket.as_deref_mut(),
                &mut lobby,
                &mut host_selection,
                &mut commands,
                &mut next_app_state,
                session.is_some(),
            );
        }
        _ => {}
    }
}
