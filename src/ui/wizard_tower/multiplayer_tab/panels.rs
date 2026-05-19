//! Top-level dispatcher that builds the multiplayer tab's left + right panels
//! based on the current `LobbyPhase`. Per-phase builders live in sibling files.

use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;

use super::panel_connect::{build_connect_left, build_connect_right};
use super::panel_failed::{build_failed_left, build_failed_right};
use super::panel_handshake::{build_handshake_left, build_handshake_right};
use super::panel_hosting::{build_hosting_left, build_hosting_right};
use super::panel_joining::{build_joining_left, build_joining_right};
use super::panel_wizard_select::{build_wizard_select_left, build_wizard_select_right};
use super::state::{LobbyPhase, MultiplayerLobby};

/// Spawns left + right panel content for the multiplayer tab based on current lobby phase.
///
/// The caller (`rebuild_panels_on_tab_change` or `rebuild_multiplayer_on_lobby_change`)
/// has already despawned existing children from both panels.
pub(crate) fn build_multiplayer_panels(
    commands: &mut Commands,
    left_entity: Entity,
    right_entity: Entity,
    lobby: &MultiplayerLobby,
    connection: &NetworkConnection,
) {
    match &lobby.phase {
        LobbyPhase::Connect => {
            build_connect_left(commands, left_entity, lobby.use_relay);
            build_connect_right(commands, right_entity, connection);
        }
        LobbyPhase::Hosting => {
            build_hosting_left(commands, left_entity);
            build_hosting_right(commands, right_entity, connection);
        }
        LobbyPhase::Joining => {
            build_joining_left(commands, left_entity);
            build_joining_right(commands, right_entity, lobby);
        }
        LobbyPhase::Handshake => {
            build_handshake_left(commands, left_entity);
            build_handshake_right(commands, right_entity, connection);
        }
        LobbyPhase::WizardSelect {
            my_wizard_types,
            my_wizard,
            opponent_wizard,
            my_ready,
            opponent_ready,
            ..
        } => {
            build_wizard_select_left(
                commands,
                left_entity,
                my_wizard_types,
                *my_wizard,
                *my_ready,
            );
            build_wizard_select_right(
                commands,
                right_entity,
                *my_wizard,
                *opponent_wizard,
                *my_ready,
                *opponent_ready,
                connection,
            );
        }
        LobbyPhase::Failed { reason } => {
            build_failed_left(commands, left_entity);
            build_failed_right(commands, right_entity, reason);
        }
    }
}

pub(super) fn spawn_ping_row(parent: &mut ChildSpawnerCommands, ping_ms: f32) {
    use crate::ui::constants::SUCCESS_COLOR;
    parent.spawn((
        Text::new(format!("Ping: {:.0}ms", ping_ms)),
        TextFont::from_font_size(super::panel_styles::HINT_FONT_SIZE),
        TextColor(SUCCESS_COLOR),
        Node {
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
    ));
}
