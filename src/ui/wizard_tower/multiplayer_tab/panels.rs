//! Top-level dispatcher that builds the multiplayer tab's panels based on the
//! current `LobbyPhase`.
//!
//! Connection setup (Connect / Hosting / Joining / Handshake / Failed) lives
//! entirely in the RIGHT panel — the left panel stays empty until a connection
//! is made. Once connected (`WizardSelect`), the LEFT panel becomes the match
//! details panel and the RIGHT panel shows the player's wizard + Switch Wizard.

use bevy::prelude::*;

use crate::networking::resources::{NetworkConnection, PeerRole};

use super::panel_connect::build_connect;
use super::panel_failed::build_failed;
use super::panel_guest_mirror::build_guest_mode_mirror;
use super::panel_handshake::build_handshake;
use super::panel_hosting::build_hosting;
use super::panel_joining::build_joining;
use super::panel_steam_hosting::build_steam_hosting;
use super::panel_steam_joining::build_steam_joining;
use super::panel_wizard_select::{build_wizard_select_left, build_wizard_select_right};
use super::state::{CoopHostSelection, LobbyPhase, MultiplayerLobby};

/// Spawns panel content for the multiplayer tab based on the current lobby phase.
///
/// The caller (`rebuild_panels_on_tab_change` or `rebuild_multiplayer_on_lobby_change`)
/// has already despawned existing children from both panels.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_multiplayer_panels(
    commands: &mut Commands,
    left_entity: Entity,
    right_entity: Entity,
    lobby: &MultiplayerLobby,
    connection: &NetworkConnection,
    steam_available: bool,
    // `false` → the Multiplayer (connection) tab; `true` → the VS (duel-setup) tab.
    for_vs_tab: bool,
    // Guest-side mirror of the host's selected mode (drives the guest's left panel).
    host_selection: Option<&CoopHostSelection>,
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

    if for_vs_tab {
        // VS tab: the 1v1 duel setup, meaningful only once connected (the lobby
        // reaches `WizardSelect`). Otherwise a "connect first" hint (the tab is
        // normally disabled until connected, so this is just a safety fallback).
        match &lobby.phase {
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
            _ => build_not_connected_hint(commands, right_entity),
        }
    } else {
        // Multiplayer (connection) tab.
        match &lobby.phase {
            LobbyPhase::Connect => {
                build_connect(
                    commands,
                    right_entity,
                    connection,
                    lobby.use_relay,
                    steam_available,
                );
            }
            LobbyPhase::Hosting => build_hosting(commands, right_entity, connection),
            LobbyPhase::Joining => build_joining(commands, right_entity, lobby),
            LobbyPhase::SteamHosting => build_steam_hosting(commands, right_entity),
            LobbyPhase::SteamJoining => build_steam_joining(commands, right_entity),
            LobbyPhase::Handshake => build_handshake(commands, right_entity, connection),
            // Connected. The GUEST does everything here: left = a live mirror of
            // the host's selected mode + Ready/Disconnect, right = switch wizard.
            // The HOST instead picks the mode on the Endless/Roguelite/VS tabs, so
            // its connection tab just points there.
            LobbyPhase::WizardSelect {
                my_wizard,
                my_ready,
                ..
            } => {
                if connection.role == Some(PeerRole::Guest) {
                    build_guest_mode_mirror(
                        commands,
                        left_entity,
                        host_selection,
                        *my_wizard,
                        *my_ready,
                        connection,
                    );
                    build_wizard_select_right(commands, right_entity, *my_wizard, *my_ready);
                } else {
                    build_connected_info(commands, right_entity);
                }
            }
            LobbyPhase::Failed { reason } => build_failed(commands, right_entity, reason),
        }
    }

    if let Some(message) = &lobby.status_message {
        commands.entity(right_entity).with_children(|right| {
            right.spawn((
                Text::new(message.clone()),
                TextFont::from_font_size(super::panel_styles::BODY_FONT_SIZE),
                TextColor(crate::ui::constants::ACTIVE_ACCENT),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
    }
}

/// Connection tab, while connected: point the player at the VS tab (duels) or
/// the game-mode tabs (co-op).
fn build_connected_info(commands: &mut Commands, right_entity: Entity) {
    use super::panel_styles::{BODY_FONT_SIZE, HEADING_FONT_SIZE};
    use crate::ui::constants::{SUCCESS_COLOR, TEXT_MUTED, TEXT_PRIMARY};

    commands.entity(right_entity).with_children(|p| {
        p.spawn((
            Text::new("Connected!"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(SUCCESS_COLOR),
        ));
        p.spawn((
            Text::new("You're the host — switch tabs up top to pick how to play:"),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));
        // Each line: a bolder mode label + a muted description.
        let mode_line = |p: &mut ChildSpawnerCommands, label: &str, desc: &str| {
            p.spawn((
                Text::new(label.to_string()),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(TEXT_PRIMARY),
                Node {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
            ));
            p.spawn((
                Text::new(desc.to_string()),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(TEXT_MUTED),
            ));
        };
        mode_line(p, "• VS tab", "Duel your friend head-to-head.");
        mode_line(p, "• Endless tab", "Team up in co-op through the endless waves.");
        mode_line(p, "• Roguelite tab", "Team up in co-op through a roguelite run.");

        p.spawn((
            Text::new(
                "Your friend stays on this Multiplayer screen — they pick a wizard and hit Ready, then you start the game.",
            ),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
        ));
    });
}

/// VS tab, when not connected (safety fallback — the tab is normally disabled
/// until a connection is live).
fn build_not_connected_hint(commands: &mut Commands, right_entity: Entity) {
    commands.entity(right_entity).with_children(|p| {
        p.spawn((
            Text::new("Not connected"),
            TextFont::from_font_size(super::panel_styles::HEADING_FONT_SIZE),
            TextColor(crate::ui::constants::TEXT_MUTED),
        ));
        p.spawn((
            Text::new("Connect to a friend on the Multiplayer tab first, then set up a duel here."),
            TextFont::from_font_size(super::panel_styles::BODY_FONT_SIZE),
            TextColor(crate::ui::constants::TEXT_MUTED),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));
    });
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
