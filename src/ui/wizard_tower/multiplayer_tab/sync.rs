//! Sync system: bridges NetworkConnection state → MultiplayerLobby phase.
//!
//! The interaction layer owns transitions the player drives (Host → Hosting,
//! Join → Joining, etc.). This system only handles transitions the transport
//! drives asynchronously: Hosting/Joining → Handshake on Connected, and any
//! phase → Failed on connection error.

use bevy::prelude::*;

use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::networking::session::MultiplayerSession;

use super::state::{LobbyPhase, MultiplayerLobby};

pub(crate) fn sync_lobby_with_connection(
    mut lobby: ResMut<MultiplayerLobby>,
    connection: Res<NetworkConnection>,
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
    }
}
