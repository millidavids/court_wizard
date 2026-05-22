//! Top-level dispatcher that builds the multiplayer tab's panels based on the
//! current `LobbyPhase`.
//!
//! Connection setup (Connect / Hosting / Joining / Handshake / Failed) lives
//! entirely in the RIGHT panel — the left panel stays empty until a connection
//! is made. Once connected (`WizardSelect`), the LEFT panel becomes the match
//! details panel and the RIGHT panel shows the player's wizard + Switch Wizard.

use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;

use super::panel_connect::build_connect;
use super::panel_failed::build_failed;
use super::panel_handshake::build_handshake;
use super::panel_hosting::build_hosting;
use super::panel_joining::build_joining;
use super::panel_wizard_select::{build_wizard_select_left, build_wizard_select_right};
use super::state::{LobbyPhase, MultiplayerLobby};

/// Spawns panel content for the multiplayer tab based on the current lobby phase.
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
    // The right panel node has no padding of its own (unlike the left panel,
    // which is padded in `layout/setup.rs`). Wrap content in a padded column
    // so it doesn't run edge-to-edge — matches the endless/study tabs.
    let right_inner = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(super::panel_styles::PANEL_PADDING)),
            row_gap: Val::Px(6.0),
            ..default()
        })
        .id();
    commands.entity(right_entity).add_child(right_inner);
    let right_entity = right_inner;

    match &lobby.phase {
        LobbyPhase::Connect => {
            build_connect(commands, right_entity, connection, lobby.use_relay);
        }
        LobbyPhase::Hosting => {
            build_hosting(commands, right_entity, connection);
        }
        LobbyPhase::Joining => {
            build_joining(commands, right_entity, lobby);
        }
        LobbyPhase::Handshake => {
            build_handshake(commands, right_entity, connection);
        }
        LobbyPhase::WizardSelect {
            my_wizard,
            opponent_wizard,
            my_ready,
            opponent_ready,
            ..
        } => {
            build_wizard_select_left(
                commands,
                left_entity,
                *my_wizard,
                *opponent_wizard,
                *my_ready,
                *opponent_ready,
                connection,
            );
            build_wizard_select_right(commands, right_entity, *my_wizard, *my_ready);
        }
        LobbyPhase::Failed { reason } => {
            build_failed(commands, right_entity, reason);
        }
    }

    if let Some(message) = &lobby.status_message {
        commands.entity(right_entity).with_children(|right| {
            right.spawn((
                Text::new(message.clone()),
                TextFont::from_font_size(super::panel_styles::BODY_FONT_SIZE),
                TextColor(crate::ui::constants::GOLD_ACCENT),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
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
