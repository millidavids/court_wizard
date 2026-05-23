//! Sync system: bridges NetworkConnection state → MultiplayerLobby phase.
//!
//! The interaction layer owns transitions the player drives (Host → Hosting,
//! Join → Joining, etc.). This system only handles transitions the transport
//! drives asynchronously: Hosting/Joining → Handshake on Connected, and any
//! phase → Failed on connection error.

use bevy::prelude::*;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::networking::session::MultiplayerSession;
use crate::ui::wizard_tower::wizard_cards::SelectedWizard;

use super::state::{LobbyPhase, MultiplayerLobby, load_my_unlocked_content};

pub(crate) fn sync_lobby_with_connection(
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    session: Option<Res<MultiplayerSession>>,
) {
    if matches!(lobby.phase, LobbyPhase::Failed { .. }) {
        return;
    }

    if connection.state == ConnectionState::Failed {
        let reason = connection
            .error
            .clone()
            .unwrap_or_else(|| "Connection failed".to_string());
        lobby.phase = LobbyPhase::Failed { reason };
        return;
    }

    if matches!(&lobby.phase, LobbyPhase::Hosting | LobbyPhase::Joining)
        && connection.state == ConnectionState::Connected
        && session.is_none()
    {
        lobby.phase = LobbyPhase::Handshake;
        // The clipboard feedback ("Code copied…") is no longer relevant once
        // the two players are connected.
        lobby.status_message = None;

        // Send our PlayerInfo exactly once — at the Handshake transition.
        // Doing it here (rather than per-frame in `process_lobby_messages`)
        // guarantees it is sent only once per connection.
        let (wizard_types, spells) = load_my_unlocked_content();
        info!(
            "[MP Lobby] Connected — sending PlayerInfo ({} wizard types, {} spells)",
            wizard_types.len(),
            spells.len()
        );
        connection
            .outgoing_messages
            .push(NetworkMessage::PlayerInfo {
                wizard_types,
                spells,
            });
    }
}

/// Bridges the shared wizard-card grid's `SelectedWizard` back into the lobby:
/// when the player picks a wizard via the grid, update `my_wizard` and tell the
/// opponent. Gated on `resource_changed::<SelectedWizard>`.
pub(crate) fn sync_mp_wizard_selection(
    selected: Res<SelectedWizard>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
) {
    let LobbyPhase::WizardSelect {
        my_wizard,
        my_ready,
        ..
    } = &mut lobby.phase
    else {
        return;
    };
    // Can't change wizard while Ready, and skip no-op reselections.
    if *my_ready || *my_wizard == Some(selected.0) {
        return;
    }
    *my_wizard = Some(selected.0);
    connection
        .outgoing_messages
        .push(NetworkMessage::WizardSelected(selected.0));
}
