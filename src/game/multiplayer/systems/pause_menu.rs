//! Multiplayer pause/escape menu, forfeit confirmation, and the escape key
//! handler that toggles pause during gameplay.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::game::resources::{GameOutcome, KillStats};
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;
use crate::networking::session::MultiplayerSession;
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::super::components::{
    MpForfeitConfirmAction, MpPauseButtonAction, OnMpForfeitConfirm, OnMpPauseScreen,
    OnMultiplayerGameScreen,
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

// ── Escape Key Handler ──────────────────────────────────────────────

/// Toggles the escape menu overlay during multiplayer gameplay.
pub(crate) fn mp_escape_key_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    action_state: Res<crate::game::input::action_state::GamepadActionState>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    // The gamepad Start button toggles the pause menu, same as Escape (mirrors the
    // single-player `keyboard_input` handler).
    let gamepad_start = active.is_gamepad()
        && action_state.just_pressed(crate::game::input::action_state::GamepadAction::Pause);
    if !keyboard.just_pressed(KeyCode::Escape) && !gamepad_start {
        return;
    }

    let Some(state) = mp_state else { return };

    // This handler only runs in Running or Paused (its run condition is
    // `in_mp_running.or(in_mp_paused)`); the SpellBook/CauldronMenu/Settings
    // overlays each close themselves back to Running/Paused via their own plugins'
    // escape handlers (`escape_to_running`, `mp_escape_to_paused`). In a co-op
    // (non-Urgent) match the synchronized `coop_pause_input` owns this toggle
    // instead, so step aside there.
    let coop_sync = session.as_ref().is_some_and(|s| s.coop_pause_synced());
    if coop_sync {
        return;
    }

    match *state.get() {
        MultiplayerGameState::Running => {
            next_mp_state.set(MultiplayerGameState::Paused);
        }
        MultiplayerGameState::Paused => {
            next_mp_state.set(MultiplayerGameState::Running);
        }
        _ => {}
    }
}

// ── Escape Menu (Paused Overlay) ────────────────────────────────────

/// Spawns the MP escape menu overlay.
///
/// Forfeit is versus-only. There is nothing to forfeit in co-op — both players are on
/// the same side — and the button was worse than useless there: the guest's
/// `NetworkMessage::Forfeit` could never be received, because `receive_mp_forfeit` is
/// gated on `MultiplayerGameState` existing and the co-op host plays in
/// `AppState::InGame`, where that sub-state does not exist. Nothing drained the
/// message either, so it sat in the host's `incoming_messages` forever. A co-op guest
/// who wants out uses Disconnect, which is right below it.
pub(crate) fn setup_mp_pause_menu(
    mut commands: Commands,
    session: Option<Res<MultiplayerSession>>,
) {
    let show_forfeit = !session.is_some_and(|s| s.is_coop());
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
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            GlobalZIndex(500),
            OnMpPauseScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Menu"),
                TextFont::from_font_size(40.0),
                TextColor(TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            spawn_button(
                parent,
                "Resume",
                MpPauseButtonAction::Resume,
                &PAUSE_BUTTON_STYLE,
            );
            spawn_button(
                parent,
                "Settings",
                MpPauseButtonAction::Settings,
                &PAUSE_BUTTON_STYLE,
            );
            if show_forfeit {
                spawn_button(
                    parent,
                    "Forfeit",
                    MpPauseButtonAction::Forfeit,
                    &PAUSE_BUTTON_STYLE,
                );
            }
            spawn_button(
                parent,
                "Disconnect",
                MpPauseButtonAction::Disconnect,
                &PAUSE_BUTTON_STYLE,
            );
        });
}

/// Cleans up the MP escape menu overlay.
pub(crate) fn cleanup_mp_pause_menu(
    mut commands: Commands,
    entities: Query<Entity, With<OnMpPauseScreen>>,
) {
    for entity in &entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }
}

/// Handles MP escape menu button clicks.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_mp_pause_buttons(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&MpPauseButtonAction>,
    mut connection: ResMut<NetworkConnection>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    transport: Option<Res<crate::networking::transport::TransportHandle>>,
    mut commands: Commands,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut lobby: ResMut<crate::ui::wizard_tower::MultiplayerLobby>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    coop_pause: Option<Res<super::super::coop_pause::CoopPauseState>>,
    mut host_selection: ResMut<crate::ui::wizard_tower::CoopHostSelection>,
) {
    let session_present = session.is_some();
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MpPauseButtonAction::Resume => {
                    // In a co-op sync-pause only the initiator may resume.
                    if !super::super::coop_pause::local_can_resume(
                        session.as_deref(),
                        coop_pause.as_deref(),
                    ) {
                        continue;
                    }
                    next_mp_state.set(MultiplayerGameState::Running);
                }
                MpPauseButtonAction::Settings => {
                    next_mp_state.set(MultiplayerGameState::Settings);
                }
                MpPauseButtonAction::Forfeit => {
                    spawn_mp_forfeit_confirm(&mut commands);
                }
                MpPauseButtonAction::Disconnect => {
                    do_mp_disconnect(
                        &mut connection,
                        transport.as_deref(),
                        steam_client.as_deref(),
                        steam_lobby.as_deref_mut(),
                        steam_socket.as_deref_mut(),
                        &mut lobby,
                        &mut host_selection,
                        &mut commands,
                        &mut next_app_state,
                        session_present,
                    );
                }
            }
        }
    }
}

/// `OnEnter(MultiplayerGameState::Paused)` after `setup_mp_pause_menu`: if the
/// guest is the NON-initiator of an active co-op sync-pause, relabel the Resume
/// button to "Waiting for other player" and mute it (the resume itself is blocked
/// in `handle_mp_pause_buttons` / `mp_escape_key_handler`).
pub(crate) fn relabel_mp_resume_for_coop(
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    pause: Option<Res<super::super::coop_pause::CoopPauseState>>,
    buttons: Query<(&MpPauseButtonAction, &Children)>,
    front_q: Query<&Children, With<crate::ui::components::ButtonFront>>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
) {
    let Some(session) = session else { return };
    let Some(pause) = pause else { return };
    if !session.coop_pause_synced() || !pause.active {
        return;
    }
    if pause.initiator == Some(session.role) {
        return;
    }
    for (action, children) in &buttons {
        if matches!(action, MpPauseButtonAction::Resume) {
            super::super::coop_pause::set_waiting_label(children, &front_q, &mut texts);
        }
    }
}

/// Spawns the forfeit confirmation overlay (sits above the pause menu).
fn spawn_mp_forfeit_confirm(commands: &mut Commands) {
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
            OnMpForfeitConfirm,
            // Also `OnMpPauseScreen` so leaving the pause menu (Escape) cleans the
            // confirmation overlay up alongside the menu.
            OnMpPauseScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Forfeit the match?\nYour opponent wins."),
                TextFont::from_font_size(32.0),
                TextColor(TEXT_PRIMARY),
                TextLayout::justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "Forfeit",
                        MpForfeitConfirmAction::Confirm,
                        &PAUSE_BUTTON_STYLE,
                    );
                    spawn_button(
                        row,
                        "Cancel",
                        MpForfeitConfirmAction::Cancel,
                        &PAUSE_BUTTON_STYLE,
                    );
                });
        });
}

/// Handles the forfeit confirmation Yes/No. The host forfeits authoritatively
/// (guest wins); the guest tells the host and returns to Running so the host's
/// `GameOver` drives its normal score-screen transition.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_mp_forfeit_confirm(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&MpForfeitConfirmAction>,
    overlay: Query<Entity, With<OnMpForfeitConfirm>>,
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    session: Res<MultiplayerSession>,
    kill_stats: Res<KillStats>,
    local_stats: Res<crate::game::multiplayer::score_stats::LocalWizardStats>,
) {
    use crate::networking::protocol::GameOverResult;
    use crate::networking::resources::PeerRole;
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };
        match action {
            MpForfeitConfirmAction::Cancel => {}
            MpForfeitConfirmAction::Confirm => {
                if session.role == PeerRole::Host {
                    super::super::host_systems::end_mp_match(
                        GameOverResult::GuestWins,
                        &mut commands,
                        &mut connection,
                        &mut game_outcome,
                        &mut next_mp_state,
                        &kill_stats,
                        &local_stats,
                    );
                } else {
                    connection.outgoing_messages.push(NetworkMessage::Forfeit);
                    next_mp_state.set(MultiplayerGameState::Running);
                }
            }
        }
        // Either choice closes the confirmation overlay.
        for e in &overlay {
            if let Ok(mut ec) = commands.get_entity(e) {
                ec.try_despawn();
            }
        }
    }
}
