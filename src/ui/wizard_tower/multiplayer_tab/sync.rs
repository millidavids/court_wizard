//! Sync system: bridges NetworkConnection state → MultiplayerLobby phase.
//!
//! The interaction layer owns transitions the player drives (Host → Hosting,
//! Join → Joining, etc.). This system only handles transitions the transport
//! drives asynchronously: Hosting/Joining → Handshake on Connected, and any
//! phase → Failed on connection error.

use bevy::prelude::*;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::ui::wizard_tower::wizard_cards::SelectedWizard;

use super::state::{CoopHostSelection, LobbyPhase, MultiplayerLobby, load_my_unlocked_content};

pub(crate) fn sync_lobby_with_connection(
    mut commands: Commands,
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

    // The Failed panel is a terminal screen — normally we stop here and wait for the
    // player to hit Retry/Back. But a fresh connection attempt can begin UNDERNEATH
    // it (most commonly a Steam invite accepted while the previous failure is still
    // on screen). If we kept returning early, that attempt would run to completion
    // with the UI deaf to it: the transport reaches `Connected`, but `Failed` is not
    // one of the four phases the Handshake transition below accepts, so `PlayerInfo`
    // — sent exactly once, from there and nowhere else — is never sent, and both
    // peers hang forever on mismatched screens.
    //
    // A live transport state therefore supersedes the stale failure: drop back to
    // `Connect` and fall through so the normal Connect → SteamJoining → Handshake
    // chain runs.
    if matches!(lobby.phase, LobbyPhase::Failed { .. }) {
        if matches!(
            connection.state,
            ConnectionState::WaitingForSignaling
                | ConnectionState::Connecting
                | ConnectionState::Connected
        ) {
            info!(
                "[MP Lobby] New connection attempt ({:?}) started under the Failed panel — resuming",
                connection.state
            );
            lobby.phase = LobbyPhase::Connect;
            lobby.status_message = None;
            *saw_active = false;
        } else {
            return;
        }
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
    // `role.is_some()` separates "the link died under us" from "we deliberately
    // tore the session down". `connection.reset()` nulls the role, so a caller
    // that just reset to baseline and moved the player straight to another phase
    // (Join Game does exactly this) is not mistaken for a dropped connection —
    // `saw_active` is still true from the session that was just torn down, and
    // without this the player lands on "Connection lost" instead of the code box.
    if connection.state == ConnectionState::Disconnected
        && *saw_active
        && connection.role.is_some()
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
    // `Connected` belongs in this set, not just the in-flight states. The resume
    // path above drops a stalled guest back to `Connect` precisely so this chain
    // can run — but if the transport finished connecting while the Failed panel
    // was up, the guest arrives here already `Connected`. Without `Connected`
    // here it never reaches `SteamJoining`, and the `-> Handshake` block below
    // does not accept `Connect`, so the guest parks at `Connect` + `Connected`
    // forever: it never sends `PlayerInfo` (the only place it is ever sent), and
    // `budget_for(PhaseKind::Connect, _)` is `None`, so nothing times it out.
    // The host sits at "Exchanging player info…" until its own handshake budget
    // expires, and every retry fails the same way until the guest is restarted.
    if matches!(lobby.phase, LobbyPhase::Connect)
        && connection.mode == ConnectionMode::Steam
        && connection.role == Some(PeerRole::Guest)
        && matches!(
            connection.state,
            ConnectionState::WaitingForSignaling
                | ConnectionState::Connecting
                | ConnectionState::Connected
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
    {
        // A `MultiplayerSession` here is definitionally stale: the four phases above
        // are all strictly pre-handshake, and a live session always sits in
        // `WizardSelect` or a match state. This used to be a `session.is_none()`
        // guard on the whole branch, which meant a leaked session silently skipped
        // the transition — and since this is the only place `PlayerInfo` is ever
        // sent, both peers deadlocked on "Connecting…" with no error and no
        // timeout. Drop it loudly and carry on; the warning is the signal that some
        // teardown path missed `reset_multiplayer_to_baseline`.
        if session.is_some() {
            warn!(
                "[MP Lobby] Stale MultiplayerSession at Connected+{:?} — dropping it",
                lobby.phase
            );
            commands.remove_resource::<MultiplayerSession>();
        }

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

// `broadcast_host_mode_to_guest` lives in `host_mode_broadcast.rs` — it is a
// host→guest data mirror, not a transport-state → phase transition.
