//! Sync system: bridges NetworkConnection state → MultiplayerLobby phase.
//!
//! The interaction layer owns transitions the player drives (Host → Hosting,
//! Join → Joining, etc.). This system only handles transitions the transport
//! drives asynchronously: Hosting/Joining → Handshake on Connected, and any
//! phase → Failed on connection error.

use bevy::prelude::*;

use crate::networking::protocol::{HostMode, NetworkMessage};
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::ui::wizard_tower::wizard_cards::SelectedWizard;

use super::state::{CoopHostSelection, LobbyPhase, MultiplayerLobby, load_my_unlocked_content};
use crate::config::GameConfig;
use crate::game::game_mode::components::{RogueliteModifiers, RogueliteRunState};
use crate::ui::wizard_tower::layout::WizardTowerTab;
use crate::ui::wizard_tower::roguelite_tab::{PendingToggles, roguelite_summary_lines};

pub(crate) fn sync_lobby_with_connection(
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    session: Option<Res<MultiplayerSession>>,
    // Local: have we ever seen the transport reach a "live" state for the
    // current lobby session? Set once `state` becomes `Connecting`,
    // `WaitingForSignaling`, or `Connected`; reset when phase returns to
    // `Connect` (i.e. a fresh attempt). Used to distinguish "Disconnected
    // is the uninitialised default" from "Disconnected after the transport
    // came up and then went away" without losing the latter case for
    // `Hosting`/`Joining`.
    mut saw_active: Local<bool>,
    mut host_selection: ResMut<CoopHostSelection>,
) {
    // The host-mode mirror is only meaningful while connected in WizardSelect.
    // Clear it in any other phase (disconnect → Failed, back to Connect, etc.) so
    // a stale mode from a previous session never lingers in the guest's panel.
    if !matches!(lobby.phase, LobbyPhase::WizardSelect { .. })
        && *host_selection != CoopHostSelection::default()
    {
        *host_selection = CoopHostSelection::default();
    }

    // Reset the lifecycle flag whenever the player is back at the Connect
    // screen (cancelled, re-entered the tab, or arrived from a recovery path).
    if matches!(lobby.phase, LobbyPhase::Connect) {
        *saw_active = false;
    }
    if matches!(
        connection.state,
        ConnectionState::Connecting
            | ConnectionState::WaitingForSignaling
            | ConnectionState::Connected
    ) {
        *saw_active = true;
    }

    if matches!(lobby.phase, LobbyPhase::Failed { .. }) {
        return;
    }

    // Transport-level errors always flip us to Failed (except from the
    // Connect screen, where the user hasn't tried to do anything yet).
    if connection.state == ConnectionState::Failed && !matches!(&lobby.phase, LobbyPhase::Connect) {
        let reason = connection
            .error
            .clone()
            .unwrap_or_else(|| "Connection failed".to_string());
        lobby.phase = LobbyPhase::Failed { reason };
        return;
    }

    // The Steam path can deliver a Failed state while the lobby UI is still
    // sitting on the Connect screen — specifically, a GameLobbyJoinRequested
    // sets mode=Steam/role=Guest/state=WaitingForSignaling without touching
    // `phase`, and `process_join_lobby_result` may flip state to Failed on the
    // same frame (e.g. friends-only mismatch, lobby already full). Without
    // this branch the previous Connect-screen guard above would skip the
    // transition and the player would be stuck on Connect with no error.
    if connection.state == ConnectionState::Failed
        && matches!(&lobby.phase, LobbyPhase::Connect)
        && connection.mode == ConnectionMode::Steam
    {
        let reason = connection
            .error
            .clone()
            .unwrap_or_else(|| "Couldn't join the Steam lobby.".to_string());
        lobby.phase = LobbyPhase::Failed { reason };
        return;
    }

    // `Disconnected` after the transport had once gone live (Connecting /
    // WaitingForSignaling / Connected) means the link is genuinely gone:
    // peer hung up, transport task died without flagging `Failed`, or the
    // loading-disconnect recovery path called `connection.reset()` while
    // the lobby was still in a post-handshake phase. The `saw_active`
    // guard prevents this from misfiring during the brief
    // `Disconnected → Connecting` window the moment the player clicks
    // Host or Join (which used to bounce them straight to a Failed panel).
    if connection.state == ConnectionState::Disconnected
        && *saw_active
        && !matches!(&lobby.phase, LobbyPhase::Connect)
    {
        let reason = connection
            .error
            .clone()
            .unwrap_or_else(|| "Connection lost".to_string());
        lobby.phase = LobbyPhase::Failed { reason };
        return;
    }

    // The Steam guest path arrives at this system with phase `Connect` (the
    // overlay click came in while we were idle on the lobby). Once the
    // GameLobbyJoinRequested handler in the Steam plugin has set mode/role
    // and started join_lobby, flip the visible phase to SteamJoining so the
    // UI matches the in-flight transport state.
    if matches!(lobby.phase, LobbyPhase::Connect)
        && connection.mode == ConnectionMode::Steam
        && connection.role == Some(PeerRole::Guest)
        && matches!(
            connection.state,
            ConnectionState::WaitingForSignaling | ConnectionState::Connecting
        )
    {
        lobby.phase = LobbyPhase::SteamJoining;
        lobby.status_message = None;
    }

    if matches!(
        &lobby.phase,
        LobbyPhase::Hosting
            | LobbyPhase::Joining
            | LobbyPhase::SteamHosting
            | LobbyPhase::SteamJoining
    ) && connection.state == ConnectionState::Connected
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
        // Send the version handshake FIRST, then PlayerInfo. The receiver
        // requires a HandshakeVersion before any other message; the order
        // here matters even on the reliable channel (FIFO).
        connection
            .outgoing_messages
            .push(NetworkMessage::HandshakeVersion {
                version: crate::networking::protocol::PROTOCOL_VERSION,
            });
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

/// Host → guest: broadcast the host's currently-selected game mode so the guest's
/// Multiplayer-tab left panel mirrors what's about to start. Runs on ANY tab (the
/// host sits on Endless/Roguelite/VS while the guest waits on Multiplayer).
///
/// Dedups on the `CoopHostSelection` descriptor (it derives `PartialEq`), and ALSO
/// re-emits when the guest first arrives (`opponent_wizard` `None→Some`) so a late
/// or reconnecting guest receives the current selection even if it hasn't changed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn broadcast_host_mode_to_guest(
    tab: Res<WizardTowerTab>,
    config: Res<GameConfig>,
    lobby: Res<MultiplayerLobby>,
    roguelite_run: Option<Res<RogueliteRunState>>,
    roguelite_modifiers: Option<Res<RogueliteModifiers>>,
    pending_toggles: Option<Res<PendingToggles>>,
    mut connection: ResMut<NetworkConnection>,
    mut last_sent: Local<Option<CoopHostSelection>>,
    mut had_guest_wizard: Local<bool>,
) {
    // Host only, while connected and in wizard-select. Reset the dedup state when
    // out of this window so a fresh lobby re-broadcasts from scratch.
    if connection.role != Some(PeerRole::Host) || connection.state != ConnectionState::Connected {
        *last_sent = None;
        *had_guest_wizard = false;
        return;
    }
    let LobbyPhase::WizardSelect {
        my_wizard,
        opponent_wizard,
        ..
    } = &lobby.phase
    else {
        *last_sent = None;
        *had_guest_wizard = false;
        return;
    };

    // Re-send when the guest first appears (`opponent_wizard` None→Some), even if
    // the descriptor is unchanged from a prior guest.
    let guest_present = opponent_wizard.is_some();
    let guest_just_arrived = guest_present && !*had_guest_wizard;
    *had_guest_wizard = guest_present;
    let host_wizard = *my_wizard;

    // Cheap scalar descriptor for the current tab. `detail_lines` are built lazily
    // below: for every mode EXCEPT the roguelite no-run tab they're fully determined
    // by these scalars, so an unchanged scalar set with no new guest is a definitive
    // no-op — skip building the (allocating) detail lines and the descriptor.
    let (mode, level, is_continue) = match *tab {
        WizardTowerTab::Endless => (
            HostMode::Endless,
            config.current_level,
            config.highest_level_achieved > 1,
        ),
        WizardTowerTab::Roguelite => match roguelite_run.as_deref() {
            Some(run) => (HostMode::Roguelite, run.level_stats.len() as u32 + 1, true),
            None => (HostMode::Roguelite, 1, false),
        },
        WizardTowerTab::Vs => (HostMode::Versus, 0, false),
        // Multiplayer / Study / anything else: not a startable mode.
        _ => (HostMode::Browsing, 0, false),
    };
    let is_roguelite_no_run = mode == HostMode::Roguelite && !is_continue;
    let scalars_match = last_sent.as_ref().is_some_and(|s| {
        s.mode == mode
            && s.level == level
            && s.is_continue == is_continue
            && s.host_wizard == host_wizard
    });
    // Only the roguelite no-run summary can change without the scalars changing
    // (the host dragging modifier sliders), so always rebuild there; otherwise an
    // unchanged scalar set means nothing to do.
    if !guest_just_arrived && scalars_match && !is_roguelite_no_run {
        return;
    }

    let detail_lines = match (mode, roguelite_run.as_deref()) {
        (HostMode::Endless, _) => vec![format!("Level {level}")],
        (HostMode::Roguelite, Some(_)) => vec![format!("Level {level} (next)")],
        (HostMode::Roguelite, None) => {
            match (roguelite_modifiers.as_deref(), pending_toggles.as_deref()) {
                (Some(m), Some(t)) => roguelite_summary_lines(m, t),
                _ => vec!["New run".to_string()],
            }
        }
        (HostMode::Versus, _) => Vec::new(),
        (HostMode::Browsing, _) => vec!["Host is choosing a mode...".to_string()],
    };

    let descriptor = CoopHostSelection {
        mode,
        host_wizard,
        level,
        is_continue,
        detail_lines,
    };

    if guest_just_arrived || last_sent.as_ref() != Some(&descriptor) {
        connection
            .outgoing_messages
            .push(NetworkMessage::HostModeSelection {
                mode: descriptor.mode,
                host_wizard: descriptor
                    .host_wizard
                    .unwrap_or(crate::config::WizardType::BoringOleMage),
                level: descriptor.level,
                is_continue: descriptor.is_continue,
                detail_lines: descriptor.detail_lines.clone(),
            });
    }
    *last_sent = Some(descriptor);
}
