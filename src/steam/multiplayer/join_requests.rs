//! Cold-start launch parameter parsing (`+connect_lobby <id>`) and the
//! `PendingSteamJoin` resource the main-menu consumer drains.

use bevy::prelude::*;
use bevy_steamworks::{Client, LobbyId};

use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::state::{AppState, MetaGameState};

use super::lobby_state::{SteamLobbyBridge, SteamLobbyState};
use super::lobby_systems::accept_incoming_join;

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
pub(super) fn parse_launch_command_at_startup(mut commands: Commands, client: Option<Res<Client>>) {
    let Some(client) = client else {
        return;
    };
    let cmdline = client.apps().launch_command_line();
    if cmdline.is_empty() {
        return;
    }
    info!("[Steam MP] Launch command line: {cmdline:?}");
    let Some(lobby_id) = parse_connect_lobby(&cmdline) else {
        return;
    };
    info!(
        "[Steam MP] Pending Steam join queued for lobby {}",
        lobby_id.raw()
    );
    commands.insert_resource(PendingSteamJoin { lobby_id });
}

/// Drain the pending join when the player reaches MainMenu or the wizard
/// tower. Sets the appropriate states, switches to the Multiplayer tab,
/// and kicks off `accept_incoming_join`.
#[allow(clippy::too_many_arguments)]
pub(super) fn consume_pending_join_in_main_menu(
    mut commands: Commands,
    pending: Option<Res<PendingSteamJoin>>,
    app_state: Res<State<AppState>>,
    client: Option<Res<Client>>,
    bridge: Option<Res<SteamLobbyBridge>>,
    mut lobby_state: Option<ResMut<SteamLobbyState>>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_meta_state: ResMut<NextState<MetaGameState>>,
) {
    let Some(pending) = pending else {
        return;
    };
    // Only consume the pending join from safe, idle states. Splash/Studio
    // must finish their animations and run their OnExit setup before we can
    // route into MetaGame, otherwise we skip MainMenu's entry-system setup
    // (audio init, leaderboard pre-fetch, save sync). MultiplayerLoading /
    // MultiplayerGame / Loading / InGame must NEVER be interrupted mid-match.
    if !matches!(app_state.get(), AppState::MainMenu | AppState::MetaGame) {
        return;
    }
    let (Some(client), Some(bridge), Some(lobby_state_mut)) = (
        client.as_deref(),
        bridge.as_deref(),
        lobby_state.as_deref_mut(),
    ) else {
        return;
    };

    let lobby_id = pending.lobby_id;
    commands.remove_resource::<PendingSteamJoin>();

    // Navigate to the wizard tower, multiplayer tab.
    next_app_state.set(AppState::MetaGame);
    next_meta_state.set(MetaGameState::WizardTower);
    commands.insert_resource(crate::ui::wizard_tower::WizardTowerTab::Multiplayer);

    accept_incoming_join(client, bridge, lobby_state_mut, &mut connection, lobby_id);
    // Make sure we're not stuck in a stale Connected state from a previous match.
    if connection.state == ConnectionState::Connected {
        connection.state = ConnectionState::WaitingForSignaling;
    }
}

/// Extract the `<u64>` that follows `+connect_lobby` in a Steam launch
/// command line. Returns `None` if the token is absent or malformed.
fn parse_connect_lobby(cmdline: &str) -> Option<LobbyId> {
    let mut tokens = cmdline.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "+connect_lobby" {
            let raw = tokens.next()?.parse::<u64>().ok()?;
            return Some(LobbyId::from_raw(raw));
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
}
