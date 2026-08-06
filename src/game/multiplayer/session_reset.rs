//! The single canonical "return multiplayer to a cold-boot baseline" helper.
//!
//! Before this existed there were seven independent teardown paths (lobby
//! Cancel/Retry, WizardTower exit, `do_mp_disconnect`, the two disconnect
//! detectors, the Steam-invite router, and quitting to the main menu), each
//! resetting a *different subset* of the multiplayer state. Whatever a given
//! path forgot became a silent deadlock on the next connection attempt, because
//! nothing in the connect flow has a timeout to surface it:
//!
//! - a leftover `MultiplayerSession` blocks the `→ Handshake` transition in
//!   `sync_lobby_with_connection`, which is the ONLY place `PlayerInfo` is sent;
//! - a leftover `LobbyPhase::Failed` makes that same system bail out early, so a
//!   fresh join completes underneath a UI that has stopped listening;
//! - a leftover `ConnectionState::Connected` makes `accept_incoming_join` refuse
//!   every subsequent Steam invite.
//!
//! Every teardown path now funnels through [`reset_multiplayer_to_baseline`], so
//! there is exactly one list to keep correct. The regression test at the bottom
//! is what stops an eighth path from drifting away from it.

use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;
use crate::networking::session::MultiplayerSession;
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::steam::multiplayer::{SteamLobbyState, SteamP2pSocket, shutdown_steam_session};
use crate::ui::wizard_tower::{CoopHostSelection, MultiplayerLobby};

use super::components::PendingRematch;
use super::coop::{
    CoopGuestConnected, CoopGuestLevel, CoopLoadingSync, CoopPeerInfo, CoopPendingSession,
};
use super::coop_pause::CoopPauseState;

/// Returns the multiplayer stack to its cold-boot baseline. Idempotent — safe to
/// call when nothing is connected, when Steam is not initialized, and twice in a
/// row.
///
/// `reason` is a short caller tag; it goes into the one-line `[MP] Reset to
/// baseline` log alongside the full pre-reset state tuple, so a future
/// stuck-connection report is diagnosable from the log alone.
///
/// **Ordering is load-bearing.** Steam teardown must run BEFORE
/// `connection.reset()`: `reset` zeroes `mode` back to `Online`, after which
/// nothing downstream can tell this was a Steam session.
///
/// `session_present` is passed in rather than read here because the callers hold
/// `MultiplayerSession` in wildly different ways (`Option<Res<_>>`, not at all,
/// or already borrowed); it is used only for the log line.
#[allow(clippy::too_many_arguments)]
pub(crate) fn reset_multiplayer_to_baseline(
    reason: &str,
    commands: &mut Commands,
    connection: &mut NetworkConnection,
    lobby: &mut MultiplayerLobby,
    host_selection: &mut CoopHostSelection,
    transport: Option<&TransportHandle>,
    steam_client: Option<&bevy_steamworks::Client>,
    steam_lobby: Option<&mut SteamLobbyState>,
    steam_socket: Option<&mut SteamP2pSocket>,
    session_present: bool,
) {
    info!(
        "[MP] Reset to baseline ({reason}): phase={:?} state={:?} mode={:?} steam_lobby={:?} session={session_present}",
        lobby.phase,
        connection.state,
        connection.mode,
        steam_lobby.as_deref(),
    );

    // 1. Signal the iroh runtime. A `Disconnect` with nothing in flight is a
    //    documented no-op (`transport/connection/runtime_loop.rs`), so this is
    //    safe to send unconditionally rather than guessing whether iroh was live.
    if let Some(transport) = transport {
        transport.send_command(TransportCommand::Disconnect);
    }

    // 2. Steam: leave the lobby, clear rich presence, drop the P2P socket.
    //    No-op on `Idle` / when Steam isn't initialized.
    shutdown_steam_session(steam_client, steam_lobby, steam_socket);

    // 3. Now the connection itself (see the ordering note above).
    connection.reset();

    // 4. The lobby UI. `MultiplayerLobby::new()` also clears
    //    `peer_protocol_version`, which the old `cancel_host_connection` leaked —
    //    a stale peer version silently disarms the "HandshakeVersion must come
    //    first" guard on the next connection.
    //
    //    `use_relay` is a persistent user preference, not session state, so it is
    //    carried across: `new()` forces it back to `true`, and silently flipping a
    //    player's LAN toggle on every Retry would be a regression.
    let use_relay = lobby.use_relay;
    *lobby = MultiplayerLobby::new();
    lobby.use_relay = use_relay;

    *host_selection = CoopHostSelection::default();

    // 5. Every per-session resource. Resources created with `init_resource` are
    //    reset to default rather than removed, because their owning plugins'
    //    run conditions expect them to always exist.
    commands.remove_resource::<MultiplayerSession>();
    commands.remove_resource::<PendingRematch>();
    commands.remove_resource::<CoopPendingSession>();
    commands.remove_resource::<CoopGuestLevel>();
    commands.remove_resource::<CoopGuestConnected>();
    commands.remove_resource::<CoopLoadingSync>();
    // Seeded from the host's `StartGame`; harmless if stale, but it is session
    // state and belongs on the same list.
    commands.remove_resource::<crate::game::seeded_rng::resources::GameRng>();
    commands.remove_resource::<crate::game::seeded_rng::resources::GameSeed>();
    commands.insert_resource(CoopPeerInfo::default());
    commands.insert_resource(CoopPauseState::default());
}

/// `OnEnter(AppState::MainMenu)`: the catch-all that closes the "quit to menu
/// never touched multiplayer" hole.
///
/// Reaching the main menu means no match and no lobby is live, but four separate
/// routes get here without any multiplayer teardown — the SP pause menu's Exit and
/// `abandon_run_for_steam_invite` (both via `abandon_run_to_main_menu`), and the SP
/// score screen's Return to Menu / End Run plus the Wizard Tower back button and
/// Escape (which set `AppState::MainMenu` directly).
///
/// That matters most for the **co-op host**, which plays in `AppState::InGame` and
/// so uses the single-player quit buttons exclusively. Without this hook it landed
/// back in the tower still owning a Steam lobby, and the "Invite Friend on Steam"
/// button then hit its non-Idle guard and silently did nothing — unrecoverable
/// without restarting the game.
///
/// Registered with `run_if(not(resource_exists::<PendingRematch>))`: the rematch
/// flow deliberately routes `MultiplayerGame → MainMenu → MetaGame` carrying a live
/// connection and session, and must be exempt. Bevy runs `OnEnter(AppState::_)`
/// strictly before `OnEnter(MenuState::Landing)` (sub-state enter schedules are
/// registered `.after` the parent's), so this lands before
/// `route_pending_rematch_from_menu` either way.
#[allow(clippy::too_many_arguments)]
pub(crate) fn reset_multiplayer_on_main_menu(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut host_selection: ResMut<CoopHostSelection>,
    transport: Option<Res<TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<SteamLobbyState>>,
    mut steam_socket: Option<ResMut<SteamP2pSocket>>,
    session: Option<Res<MultiplayerSession>>,
) {
    reset_multiplayer_to_baseline(
        "main menu entered",
        &mut commands,
        &mut connection,
        &mut lobby,
        &mut host_selection,
        transport.as_deref(),
        steam_client.as_deref(),
        steam_lobby.as_deref_mut(),
        steam_socket.as_deref_mut(),
        session.is_some(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::networking::resources::{ConnectionMode, ConnectionState, PeerRole};
    use crate::networking::session::SessionMode;
    use crate::ui::wizard_tower::LobbyPhase;
    use bevy::ecs::system::RunSystemOnce;

    /// Deliberately dirty every piece of state a session can leave behind, run the
    /// helper, and assert we are back at a cold-boot baseline.
    ///
    /// Built on a bare `App::new()` with no plugins — `DefaultPlugins` would try to
    /// open a window. Steam resources are absent, which also exercises the
    /// "Steam not initialized" path.
    #[test]
    fn resets_every_per_session_resource() {
        let mut app = App::new();

        app.insert_resource(NetworkConnection {
            state: ConnectionState::Connected,
            role: Some(PeerRole::Guest),
            mode: ConnectionMode::Steam,
            local_code: Some("stale".to_string()),
            error: Some("stale error".to_string()),
            ..Default::default()
        });

        app.insert_resource(MultiplayerLobby {
            phase: LobbyPhase::Failed {
                reason: "stale".to_string(),
            },
            peer_protocol_version: Some(13),
            join_code_input: "stale-code".to_string(),
            // A non-default preference, to prove it survives the reset.
            use_relay: false,
            ..MultiplayerLobby::new()
        });

        app.insert_resource(CoopHostSelection {
            level: 7,
            ..Default::default()
        });
        app.insert_resource(MultiplayerSession {
            role: PeerRole::Guest,
            mode: SessionMode::CoopEndless,
            host_wizard: crate::config::WizardType::BoringOleMage,
            guest_wizard: crate::config::WizardType::BoringOleMage,
            host_spells: Vec::new(),
            guest_spells: Vec::new(),
            coop_urgent: false,
        });
        app.insert_resource(PendingRematch);
        app.insert_resource(CoopGuestLevel(4));
        app.insert_resource(CoopGuestConnected(true));
        app.insert_resource(CoopLoadingSync {
            my_loaded: true,
            peer_loaded: true,
        });

        app.world_mut()
            .run_system_once(
                |mut commands: Commands,
                 mut connection: ResMut<NetworkConnection>,
                 mut lobby: ResMut<MultiplayerLobby>,
                 mut host_selection: ResMut<CoopHostSelection>| {
                    reset_multiplayer_to_baseline(
                        "test",
                        &mut commands,
                        &mut connection,
                        &mut lobby,
                        &mut host_selection,
                        None,
                        None,
                        None,
                        None,
                        true,
                    );
                },
            )
            .expect("reset system should run");

        let world = app.world();

        let connection = world.resource::<NetworkConnection>();
        assert_eq!(connection.state, ConnectionState::Disconnected);
        assert_eq!(connection.role, None);
        assert_eq!(connection.mode, ConnectionMode::Online);
        assert_eq!(connection.local_code, None);
        assert_eq!(connection.error, None);

        let lobby = world.resource::<MultiplayerLobby>();
        assert_eq!(lobby.phase, LobbyPhase::Connect);
        assert_eq!(lobby.peer_protocol_version, None);
        assert!(lobby.join_code_input.is_empty());
        assert!(
            !lobby.use_relay,
            "use_relay is a preference, not session state"
        );

        assert_eq!(
            *world.resource::<CoopHostSelection>(),
            CoopHostSelection::default()
        );

        assert!(world.get_resource::<MultiplayerSession>().is_none());
        assert!(world.get_resource::<PendingRematch>().is_none());
        assert!(world.get_resource::<CoopPendingSession>().is_none());
        assert!(world.get_resource::<CoopGuestLevel>().is_none());
        assert!(world.get_resource::<CoopGuestConnected>().is_none());
        assert!(world.get_resource::<CoopLoadingSync>().is_none());
        assert!(!world.resource::<CoopPauseState>().active);
        assert!(world.resource::<CoopPeerInfo>().name.is_none());
    }

    /// Calling the helper on already-clean state must not panic or resurrect
    /// anything — several call sites can fire twice in quick succession.
    #[test]
    fn is_idempotent() {
        let mut app = App::new();
        app.insert_resource(NetworkConnection::default());
        app.insert_resource(MultiplayerLobby::new());
        app.insert_resource(CoopHostSelection::default());

        for _ in 0..2 {
            app.world_mut()
                .run_system_once(
                    |mut commands: Commands,
                     mut connection: ResMut<NetworkConnection>,
                     mut lobby: ResMut<MultiplayerLobby>,
                     mut host_selection: ResMut<CoopHostSelection>| {
                        reset_multiplayer_to_baseline(
                            "test",
                            &mut commands,
                            &mut connection,
                            &mut lobby,
                            &mut host_selection,
                            None,
                            None,
                            None,
                            None,
                            false,
                        );
                    },
                )
                .expect("reset system should run");
        }

        assert_eq!(*app.world().resource::<MultiplayerLobby>(), {
            MultiplayerLobby::new()
        });
    }
}
