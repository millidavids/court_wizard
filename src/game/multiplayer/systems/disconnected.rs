//! Multiplayer disconnected overlay UI and connection-loss detection systems.

use bevy::prelude::*;

use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::networking::session::MultiplayerSession;
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::super::components::{
    MpDisconnectedButtonAction, OnMpDisconnectedScreen, OnMultiplayerGameScreen,
};
use super::lifecycle::do_mp_disconnect;

const PAUSE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

// ── Disconnected Overlay ────────────────────────────────────────────

/// Spawns the disconnected overlay informing the player of connection loss.
pub(crate) fn setup_mp_disconnected(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.85)),
            GlobalZIndex(600),
            OnMpDisconnectedScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Disconnected"),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(0.95, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new("The connection to your opponent was lost."),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            spawn_button(
                parent,
                "Return to Menu",
                MpDisconnectedButtonAction,
                &PAUSE_BUTTON_STYLE,
            );
        });
}

/// Cleans up the disconnected overlay.
pub(crate) fn cleanup_mp_disconnected(
    mut commands: Commands,
    entities: Query<Entity, With<OnMpDisconnectedScreen>>,
) {
    for entity in &entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }
}

/// Handles the Return to Menu button on the disconnected overlay.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_mp_disconnected_buttons(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&MpDisconnectedButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut lobby: ResMut<crate::ui::wizard_tower::MultiplayerLobby>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            // `None` transport: the overlay only appears when the peer is already
            // gone, so there's nothing to signal — just tear down locally.
            do_mp_disconnect(
                &mut connection,
                None,
                steam_client.as_deref(),
                steam_lobby.as_deref_mut(),
                steam_socket.as_deref_mut(),
                &mut lobby,
                &mut commands,
                &mut next_app_state,
            );
            return;
        }
    }
}

/// Detects unexpected connection loss and transitions to the Disconnected overlay.
///
/// Only triggers on `Failed` state — intentional disconnects are handled by button actions.
pub(crate) fn detect_mp_disconnect(
    mut commands: Commands,
    connection: Res<NetworkConnection>,
    session: Option<Res<MultiplayerSession>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    mut next_app_state: ResMut<NextState<crate::state::AppState>>,
    mut notifications: ResMut<crate::ui::notification::NotificationQueue>,
) {
    // Only check if we still have an active session — avoids double-triggering
    // after an intentional disconnect already queued a state transition.
    let Some(session) = session else {
        return;
    };

    // Don't re-trigger if already on the Disconnected screen
    if mp_state.is_some_and(|s| *s.get() == MultiplayerGameState::Disconnected) {
        return;
    }

    // Both Failed (transport error) and Disconnected (peer-initiated graceful
    // close) are treated as unexpected loss here. The transport emits
    // `StateChanged(Disconnected)` after the peer closes cleanly, and without
    // this we'd silently leave the player stuck in a live match with a dead
    // socket.
    if !matches!(
        connection.state,
        ConnectionState::Failed | ConnectionState::Disconnected
    ) {
        return;
    }

    if session.is_coop() {
        // Co-op guest: the host keeps playing solo, so there's no "match over"
        // dead-end for the guest. Drop the per-match co-op state and return to
        // the wizard tower's Multiplayer tab, where the guest can rejoin (the
        // host's endpoint stays bound and re-listens — see the transport
        // re-accept loop). Clearing the session lets a fresh `StartGame` build
        // a clean one on rejoin.
        commands.remove_resource::<MultiplayerSession>();
        commands.remove_resource::<crate::game::multiplayer::coop::CoopGuestLevel>();
        // The co-op guest returns silently to the tower, so flag the reason with a
        // toast — otherwise it's not obvious the host dropped the game.
        notifications.push(crate::ui::notification::NotificationEntry::Toast {
            message: "The host disconnected.",
        });
        next_app_state.set(crate::state::AppState::MetaGame);
    } else {
        next_mp_state.set(MultiplayerGameState::Disconnected);
    }
}

/// Detects connection loss while we're still in `MultiplayerLoading` (before
/// the `MultiplayerGame` sub-state machine exists) and bails out cleanly.
///
/// Without this, a disconnect during loading would let `process_mp_spawn_queue`
/// keep ticking against a dead connection until the queue completed and the
/// game transitioned to `MultiplayerGame` — at which point `detect_mp_disconnect`
/// would finally fire. This system shortcuts that by sending the player
/// straight back to the wizard tower with the connection torn down.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_mp_loading_disconnect(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    session: Option<Res<MultiplayerSession>>,
    transport: Option<Res<TransportHandle>>,
    mut next_app_state: ResMut<NextState<crate::state::AppState>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    // Despawn any battlefield, units, terrain, or screen-tagged UI that the
    // spawn queue produced before the disconnect hit. Without these the 3D
    // meshes leak into the wizard tower scene after we transition.
    gameplay_entities: Query<
        Entity,
        Or<(
            With<crate::game::components::OnGameplayScreen>,
            With<super::super::components::OnMultiplayerGameScreen>,
        )>,
    >,
) {
    // Mirror `detect_mp_disconnect`'s guard: a missing session means we're
    // not actually in a live match — the `Disconnected` default-state could
    // otherwise spuriously fire this handler on the first frame of loading.
    if session.is_none() {
        return;
    }

    if matches!(
        connection.state,
        ConnectionState::Failed | ConnectionState::Disconnected
    ) {
        warn!("[MP] Connection lost during loading; returning to wizard tower.");
        if let Some(t) = transport.as_ref() {
            t.send_command(TransportCommand::Disconnect);
        }
        // Steam teardown must run BEFORE connection.reset() so we can still
        // see `mode == Steam` and find the SteamLobbyState we need to leave.
        crate::steam::multiplayer::shutdown_steam_session(
            steam_client.as_deref(),
            steam_lobby.as_deref_mut(),
            steam_socket.as_deref_mut(),
        );
        // Clean up everything the loading queue had already spawned —
        // `OnExit(MultiplayerLoading)` only tears down the loading screen
        // and queue resources, so without these despawns the partially-built
        // arena would leak into MetaGame.
        for entity in &gameplay_entities {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.try_despawn();
            }
        }
        commands.remove_resource::<crate::game::pathfinding::resources::PathfindingGrid>();
        connection.reset();
        commands.remove_resource::<MultiplayerSession>();
        // `MetaGameState` is a SubState of `AppState::MetaGame` and is
        // re-initialised to its `#[default] = WizardTower` whenever the
        // parent state enters — no explicit `next_meta_state.set` needed.
        next_app_state.set(crate::state::AppState::MetaGame);
    }
}
