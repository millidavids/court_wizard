//! Multiplayer tab plugin — system registration only.

use bevy::prelude::*;

use crate::game::multiplayer::components::PendingRematch;
use crate::networking::resources::NetworkConnection;
use crate::networking::transport::TransportCommand;
use crate::networking::transport::TransportHandle;
use crate::state::MetaGameState;
use crate::ui::plugin::ButtonActionSet;

use super::interaction::handle_mp_tab_actions;
use super::lobby_messages::process_lobby_messages;
use super::state::{LobbyPhase, MultiplayerLobby};
use super::sync::sync_lobby_with_connection;
use super::text_input::handle_join_code_input;

/// Plugin that registers all systems for the multiplayer tab.
pub struct MultiplayerTabPlugin;

impl Plugin for MultiplayerTabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiplayerLobby>()
            .add_systems(
                OnEnter(MetaGameState::WizardTower),
                handle_pending_rematch_on_enter,
            )
            .add_systems(OnExit(MetaGameState::WizardTower), reset_lobby_on_exit)
            .add_systems(
                Update,
                (
                    handle_mp_tab_actions.in_set(ButtonActionSet),
                    handle_join_code_input,
                    process_lobby_messages,
                    sync_lobby_with_connection,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(multiplayer_tab_active),
            );
    }
}

fn multiplayer_tab_active(
    tab: Option<Res<crate::ui::wizard_tower::layout::WizardTowerTab>>,
) -> bool {
    tab.is_some_and(|t| {
        *t == crate::ui::wizard_tower::layout::WizardTowerTab::Multiplayer
    })
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

    // Pre-populate WizardSelect from the previous session
    let (my_wt, _) = super::state::load_my_unlocked_content();
    let initial = my_wt
        .first()
        .copied()
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

    // Notify the opponent of our initial selection (connection is still alive)
    connection
        .outgoing_messages
        .push(NetworkMessage::WizardSelected(initial));
}

/// When leaving WizardTower, disconnect and reset the lobby to `Connect` phase.
fn reset_lobby_on_exit(
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    transport: Option<Res<TransportHandle>>,
) {
    use crate::networking::resources::ConnectionState;
    if connection.state != ConnectionState::Disconnected {
        if let Some(t) = transport {
            t.send_command(TransportCommand::Disconnect);
        }
        connection.reset();
    }
    *lobby = MultiplayerLobby::new();
}
