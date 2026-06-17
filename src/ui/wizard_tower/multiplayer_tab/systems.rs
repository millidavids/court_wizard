//! Lifecycle systems for the multiplayer tab (rematch routing, lobby reset).

use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::multiplayer::components::PendingRematch;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection};
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::state::AppState;
use crate::steam::multiplayer::{
    SteamLobbyState, SteamP2pSocket, leave_steam_lobby, tear_down_socket,
};
use crate::ui::wizard_tower::layout::WizardTowerTab;
use crate::ui::wizard_tower::wizard_cards::SelectedWizard;

use super::state::{CoopHostSelection, LobbyPhase, MultiplayerLobby};

/// When the main-menu landing screen is entered with a `PendingRematch`
/// resource present, immediately route into the Wizard Tower so the rematch
/// lobby can be set up. Without this the player is stranded on the main menu.
pub(super) fn route_pending_rematch_from_menu(
    pending: Option<Res<PendingRematch>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if pending.is_some() {
        next_app_state.set(AppState::MetaGame);
    }
}

/// Returns `true` when the active tab is one that hosts multiplayer lobby
/// interaction (Multiplayer connection tab or the VS duel tab).
pub(super) fn mp_tab_selected(tab: Option<Res<WizardTowerTab>>) -> bool {
    tab.is_some_and(|t| matches!(*t, WizardTowerTab::Multiplayer | WizardTowerTab::Vs))
}

/// Force the Multiplayer tab active: mutate the `WizardTowerTab` resource if it
/// exists, otherwise insert it. Shared by the rematch routing and the
/// Steam-invite routing (`route_pending_steam_join`).
pub(crate) fn force_mp_tab(tab: &mut Option<ResMut<WizardTowerTab>>, commands: &mut Commands) {
    match tab.as_deref_mut() {
        Some(t) => *t = WizardTowerTab::Multiplayer,
        None => commands.insert_resource(WizardTowerTab::Multiplayer),
    }
}

/// When returning to WizardTower with a `PendingRematch`, set the Multiplayer
/// tab active and pre-populate the lobby into `WizardSelect` phase so the
/// player skips the Connect screen.
pub(super) fn handle_pending_rematch_on_enter(
    mut commands: Commands,
    pending: Option<Res<PendingRematch>>,
    mut lobby: ResMut<MultiplayerLobby>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    mut tab: Option<ResMut<WizardTowerTab>>,
    mut connection: ResMut<NetworkConnection>,
) {
    if pending.is_none() {
        return;
    }
    commands.remove_resource::<PendingRematch>();

    // Set the Multiplayer tab active
    force_mp_tab(&mut tab, &mut commands);

    // Pre-populate WizardSelect from the previous session. Keep the wizard the
    // player chose last match (the `MultiplayerSession` survives a rematch), only
    // falling back to the first unlocked type if it's somehow gone/locked.
    let (my_wt, _) = super::state::load_my_unlocked_content();
    let initial = session
        .as_ref()
        .map(|s| s.local_wizard())
        .filter(|wt| my_wt.contains(wt))
        .or_else(|| my_wt.first().copied())
        .unwrap_or(WizardType::BoringOleMage);

    let previous_opponent = session.as_ref().map(|s| {
        use crate::networking::resources::PeerRole;
        match s.role {
            PeerRole::Host => s.guest_wizard,
            PeerRole::Guest => s.host_wizard,
        }
    });

    lobby.phase = LobbyPhase::WizardSelect {
        my_wizard_types: my_wt,
        opponent_wizard_types: Vec::new(),
        my_wizard: Some(initial),
        opponent_wizard: previous_opponent,
        my_ready: false,
        opponent_ready: false,
    };
    // Seed the shared wizard-card grid's selection.
    commands.insert_resource(SelectedWizard(initial));

    // Drop any messages left over from the previous match (GameOver, stale
    // ReadyUp, etc.) so they don't mis-advance the fresh rematch lobby.
    connection.incoming_messages.clear();
    connection.outgoing_messages.clear();
    connection.incoming_unreliable.clear();
    connection.outgoing_unreliable.clear();

    // Notify the opponent of our initial selection (connection is still alive)
    connection
        .outgoing_messages
        .push(NetworkMessage::WizardSelected(initial));
}

/// When leaving WizardTower, disconnect and reset the lobby to `Connect` phase.
///
/// Starting a multiplayer match also exits WizardTower — but in that case the
/// live connection must survive into the match, so the teardown is skipped
/// when the destination is a multiplayer loading/game state.
#[allow(clippy::too_many_arguments)]
pub(super) fn reset_lobby_on_exit(
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    transport: Option<Res<TransportHandle>>,
    app_state: Res<State<AppState>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby_state: Option<ResMut<SteamLobbyState>>,
    mut steam_socket: Option<ResMut<SteamP2pSocket>>,
    coop_pending: Option<Res<crate::game::multiplayer::coop::CoopPendingSession>>,
    mut host_selection: ResMut<CoopHostSelection>,
) {
    // Preserve the connection when leaving the tower INTO a match: versus goes to
    // MultiplayerLoading/MultiplayerGame; the co-op host goes to the single-player
    // `AppState::Loading` and is identified by a pending co-op session. Without
    // the co-op case the host's connection would be torn down the instant a co-op
    // match starts (and between every endless level, which loops via the tower).
    if matches!(
        app_state.get(),
        AppState::MultiplayerLoading | AppState::MultiplayerGame
    ) || coop_pending.is_some()
    {
        return;
    }

    // Iroh teardown — only when iroh was actually live. Sending Disconnect
    // unconditionally would write a no-op into the command channel on every
    // WizardTower exit (including Steam-mode exits and exits from a tab the
    // player never touched), which changes the channel's "only written when
    // there is work to cancel" contract.
    if connection.mode != ConnectionMode::Steam
        && connection.state != ConnectionState::Disconnected
        && let Some(t) = transport
    {
        t.send_command(TransportCommand::Disconnect);
    }

    // Steam teardown: leave the lobby + clear rich presence + close the socket.
    // No-op when Steam isn't initialized.
    if connection.mode == ConnectionMode::Steam {
        if let (Some(client), Some(lobby_state)) =
            (steam_client.as_deref(), steam_lobby_state.as_deref_mut())
        {
            leave_steam_lobby(client, lobby_state);
        }
        if let Some(socket) = steam_socket.as_deref_mut() {
            tear_down_socket(socket);
        }
    }

    // Always reset — calling unconditionally clears stranded `mode == Steam`
    // flags from partially-completed flows that left `state` already
    // `Disconnected`.
    connection.reset();
    *lobby = MultiplayerLobby::new();
    *host_selection = CoopHostSelection::default();
}
