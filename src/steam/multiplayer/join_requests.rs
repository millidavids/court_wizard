//! Cold-start launch parameter parsing (`+connect_lobby <id>`) and the
//! `PendingSteamJoin` resource the main-menu consumer drains.

use bevy::prelude::*;
use bevy_steamworks::{Client, LobbyId};

use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::state::AppState;

use super::lobby_state::{SteamLobbyBridge, SteamLobbyState, shutdown_steam_session};
use super::lobby_systems::accept_incoming_join;
use super::sockets::SteamP2pSocket;

/// Inserted at Startup when Steam launched the binary with `+connect_lobby <id>`.
/// Consumed by `consume_pending_join_in_main_menu` once the player is at a
/// menu state safe to route into the multiplayer tab.
#[derive(Resource)]
pub(crate) struct PendingSteamJoin {
    pub(super) lobby_id: LobbyId,
}

/// Inspect Steam's command-line invite parameter at boot. If it's
/// `+connect_lobby <u64>`, stash the lobby id for the main-menu consumer to
/// pick up once we're past Splash / Studio screens.
///
/// A cold-launch lobby invite ("Join Game" while the game is closed) is delivered
/// by Steam as `+connect_lobby <id>` on the **OS process command line (argv)** —
/// NOT through `ISteamApps::GetLaunchCommandLine()`, which only returns the
/// connect string when the "Use launch command line" install option is enabled
/// (the rich-presence path). So we check argv FIRST, then fall back to the Steam
/// API string, so cold-launch joins work regardless of that dashboard setting.
pub(super) fn parse_launch_command_at_startup(mut commands: Commands, client: Option<Res<Client>>) {
    // Skip argv[0] (the exe path): it's never part of the invite, keeps the
    // player's filesystem path out of logs, and avoids a contrived false match if
    // the install path itself contained the token. `args_os` + lossy conversion so
    // a non-Unicode argument can't panic at boot — `std::env::args()` is documented
    // to panic on invalid Unicode, which at Startup would crash the game on launch.
    let os_cmdline = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    let steam_cmdline = client
        .as_deref()
        .map(|c| c.apps().launch_command_line())
        .unwrap_or_default();

    // Log whatever Steam handed us (only when non-empty, so normal launches stay
    // quiet) so a cold-launch that still fails to route is diagnosable from the log.
    if !os_cmdline.is_empty() || !steam_cmdline.is_empty() {
        info!("[Steam MP] Launch args (argv={os_cmdline:?}, steam_cmdline={steam_cmdline:?})");
    }

    let Some(lobby_id) =
        parse_connect_lobby(&os_cmdline).or_else(|| parse_connect_lobby(&steam_cmdline))
    else {
        return;
    };
    info!(
        "[Steam MP] Pending Steam join queued for lobby {}",
        lobby_id.raw()
    );
    commands.insert_resource(PendingSteamJoin { lobby_id });
}

/// Route an accepted Steam invite into the multiplayer tab and connect, from a
/// menu. Gate with `run_if(resource_exists::<PendingSteamJoin>)`.
///
/// This is the menu half of the routing pipeline; the active-run half lives in
/// `abandon_run_for_steam_invite` (game side), which tears a match down to the
/// main menu first. Behaviour by state:
/// - `MainMenu` → head to the wizard tower (`MetaGameState` has a single variant,
///   `WizardTower`, so entering `MetaGame` lands there automatically). Keep the
///   intent so the `MetaGame` branch finishes the join.
/// - `MetaGame`/`WizardTower` → terminal: force the Multiplayer tab, clear any
///   stale lobby we already had open, `accept_incoming_join`, and drop the intent.
/// - anything else (`Splash`/`Loading`/active run) → wait; another frame (or
///   `abandon_run_for_steam_invite`) will get us to a menu.
#[allow(clippy::too_many_arguments)]
pub(super) fn route_pending_steam_join(
    mut commands: Commands,
    pending: Res<PendingSteamJoin>,
    app_state: Res<State<AppState>>,
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut socket: Option<ResMut<SteamP2pSocket>>,
    mut tab: Option<ResMut<crate::ui::wizard_tower::WizardTowerTab>>,
) {
    match app_state.get() {
        AppState::MainMenu => {
            // Splash/Studio have finished and MainMenu's entry systems have run;
            // head into the wizard tower. The MetaGame branch (next frames) joins.
            next_app_state.set(AppState::MetaGame);
        }
        AppState::MetaGame => {
            // Steam resources may not exist for one frame at boot (the bridge is
            // built in a Startup system) — the Option guard just retries next frame.
            let (Some(client), Some(bridge), Some(lobby_state)) = (
                client.as_deref(),
                bridge.as_deref(),
                lobby_state.as_deref_mut(),
            ) else {
                return;
            };

            // Force the Multiplayer tab in Update (after `setup_wizard_tower_layout`
            // ran in OnEnter) so we beat its dormant-roguelite-run tab override.
            crate::ui::wizard_tower::force_mp_tab(&mut tab, &mut commands);

            // If we already had a lobby open (e.g. mid-host or already connected to
            // a peer in the tower), fully tear it down before joining the new one —
            // `shutdown_steam_session` leaves the lobby AND the P2P socket, matching
            // `do_mp_disconnect`/`reset_lobby_on_exit`. `accept_incoming_join` also
            // refuses on any non-Idle lobby, so the leave is required, not optional.
            if !matches!(lobby_state, SteamLobbyState::Idle) {
                shutdown_steam_session(Some(client), Some(lobby_state), socket.as_deref_mut());
                connection.reset();
            }

            accept_incoming_join(client, bridge, lobby_state, &mut connection, pending.lobby_id);
            // Make sure we're not stuck in a stale Connected state from a previous match.
            if connection.state == ConnectionState::Connected {
                connection.state = ConnectionState::WaitingForSignaling;
            }
            commands.remove_resource::<PendingSteamJoin>();
        }
        // Splash / Loading / MultiplayerLoading / InGame / MultiplayerGame: wait.
        // `abandon_run_for_steam_invite` handles tearing down an active run.
        _ => {}
    }
}

/// Extract the `<u64>` that follows `+connect_lobby` in a Steam launch command
/// line (cold-start) or rich-presence connect string (in-game friends-list join).
/// Returns `None` if the token is absent or malformed.
pub(super) fn parse_connect_lobby(cmdline: &str) -> Option<LobbyId> {
    let mut tokens = cmdline.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "+connect_lobby" {
            let raw = tokens.next()?.parse::<u64>().ok()?;
            // 0 is the invalid/empty Steam id; reject it so a truncated or
            // corrupted connect string can't route a doomed join.
            return (raw != 0).then(|| LobbyId::from_raw(raw));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_connect_lobby() {
        let id = parse_connect_lobby("court_wizard.exe +connect_lobby 12345").unwrap();
        assert_eq!(id.raw(), 12345);
    }

    #[test]
    fn absent_returns_none() {
        assert!(parse_connect_lobby("court_wizard.exe").is_none());
    }

    #[test]
    fn malformed_returns_none() {
        assert!(parse_connect_lobby("court_wizard.exe +connect_lobby NaN").is_none());
        assert!(parse_connect_lobby("court_wizard.exe +connect_lobby").is_none());
    }

    #[test]
    fn zero_lobby_id_returns_none() {
        assert!(parse_connect_lobby("court_wizard.exe +connect_lobby 0").is_none());
    }
}
