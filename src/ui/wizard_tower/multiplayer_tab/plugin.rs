//! Multiplayer tab plugin — system registration only.

use bevy::prelude::*;

use crate::game::multiplayer::components::PendingRematch;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection};
use crate::networking::transport::TransportCommand;
use crate::networking::transport::TransportHandle;
use crate::state::{AppState, MenuState, MetaGameState};
use crate::steam::multiplayer::{
    SteamLobbyState, SteamP2pSocket, leave_steam_lobby, tear_down_socket,
};
use crate::ui::plugin::ButtonActionSet;

use crate::ui::wizard_tower::wizard_cards::SelectedWizard;

use super::interaction::handle_mp_tab_actions;
use super::lobby_messages::process_lobby_messages;
use super::state::{LobbyPhase, MultiplayerLobby};
use super::sync::{sync_lobby_with_connection, sync_mp_wizard_selection};
use super::text_input::handle_join_code_input;

/// Plugin that registers all systems for the multiplayer tab.
pub struct MultiplayerTabPlugin;

impl Plugin for MultiplayerTabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiplayerLobby>()
            // A rematch routes through the main menu; bounce straight back into
            // the tower so `handle_pending_rematch_on_enter` can pick it up.
            .add_systems(OnEnter(MenuState::Landing), route_pending_rematch_from_menu)
            .add_systems(
                OnEnter(MetaGameState::WizardTower),
                handle_pending_rematch_on_enter,
            )
            .add_systems(OnExit(MetaGameState::WizardTower), reset_lobby_on_exit)
            // The lobby network pump must run on any Wizard Tower tab — not just
            // the Multiplayer tab — so a connection isn't stranded if the player
            // switches tabs mid-handshake.
            .add_systems(
                Update,
                (process_lobby_messages, sync_lobby_with_connection)
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // Tab UI interaction only runs while the Multiplayer tab is shown.
            .add_systems(
                Update,
                (
                    handle_mp_tab_actions.in_set(ButtonActionSet),
                    handle_join_code_input,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(multiplayer_tab_active),
            )
            // The wizard-selection sync needs `SelectedWizard` to exist (it is
            // inserted lazily), so it carries its own resource-gated condition.
            .add_systems(
                Update,
                sync_mp_wizard_selection
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(multiplayer_tab_active)
                    .run_if(
                        resource_exists::<SelectedWizard>.and(resource_changed::<SelectedWizard>),
                    ),
            );
    }
}

/// When the main-menu landing screen is entered with a `PendingRematch`
/// resource present, immediately route into the Wizard Tower so the rematch
/// lobby can be set up. Without this the player is stranded on the main menu.
fn route_pending_rematch_from_menu(
    pending: Option<Res<PendingRematch>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if pending.is_some() {
        next_app_state.set(AppState::MetaGame);
    }
}

fn multiplayer_tab_active(
    tab: Option<Res<crate::ui::wizard_tower::layout::WizardTowerTab>>,
) -> bool {
    use crate::ui::wizard_tower::layout::WizardTowerTab;
    // The lobby interaction (wizard select, ready-up, disconnect, start) lives on
    // BOTH the Multiplayer connection tab and the VS duel tab, so its handlers must
    // run for either — otherwise the VS tab's buttons are inert.
    tab.is_some_and(|t| matches!(*t, WizardTowerTab::Multiplayer | WizardTowerTab::Vs))
}

/// When returning to WizardTower with a `PendingRematch`, set the Multiplayer
/// tab active and pre-populate the lobby into `WizardSelect` phase so the
/// player skips the Connect screen.
fn handle_pending_rematch_on_enter(
    mut commands: Commands,
    pending: Option<Res<PendingRematch>>,
    mut lobby: ResMut<MultiplayerLobby>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    mut tab: Option<ResMut<crate::ui::wizard_tower::layout::WizardTowerTab>>,
    mut connection: ResMut<NetworkConnection>,
) {
    if pending.is_none() {
        return;
    }
    commands.remove_resource::<PendingRematch>();

    // Set the Multiplayer tab active
    if let Some(ref mut t) = tab {
        **t = crate::ui::wizard_tower::layout::WizardTowerTab::Multiplayer;
    } else {
        commands.insert_resource(crate::ui::wizard_tower::layout::WizardTowerTab::Multiplayer);
    }

    // Pre-populate WizardSelect from the previous session. Keep the wizard the
    // player chose last match (the `MultiplayerSession` survives a rematch), only
    // falling back to the first unlocked type if it's somehow gone/locked.
    let (my_wt, _) = super::state::load_my_unlocked_content();
    let initial = session
        .as_ref()
        .map(|s| s.local_wizard())
        .filter(|wt| my_wt.contains(wt))
        .or_else(|| my_wt.first().copied())
        .unwrap_or(crate::config::WizardType::BoringOleMage);

    let previous_opponent = session.as_ref().map(|s| {
        use crate::networking::resources::PeerRole;
        match s.role {
            PeerRole::Host => s.guest_wizard,
            PeerRole::Guest => s.host_wizard,
        }
    });

    use crate::networking::protocol::NetworkMessage;
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
fn reset_lobby_on_exit(
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    transport: Option<Res<TransportHandle>>,
    app_state: Res<State<AppState>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby_state: Option<ResMut<SteamLobbyState>>,
    mut steam_socket: Option<ResMut<SteamP2pSocket>>,
    coop_pending: Option<Res<crate::game::multiplayer::coop::CoopPendingSession>>,
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
}
