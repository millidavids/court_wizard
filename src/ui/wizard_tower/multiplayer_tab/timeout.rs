//! Deadlines for the lobby's connecting phases.
//!
//! Nothing in the connect flow used to have a timeout: every "Connecting…" /
//! "Waiting for…" screen was exited only by an explicit success or an explicit
//! failure event. Any stall — a Steam call-result callback that never fires, an
//! SDR handshake that never completes, a `PlayerInfo` that never arrives — left
//! the player on a dead screen with no error, which is what made the underlying
//! state bugs feel like "the game is broken, restart it".
//!
//! This converts a stalled phase into `LobbyPhase::Failed`, which is a screen the
//! player can actually act on (Try Again runs the full baseline reset). It is a
//! backstop, not the fix: if a budget is ever hit, the `warn!` names the exact
//! state tuple that stalled.

use bevy::prelude::*;

use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::steam::multiplayer::SteamLobbyState;

use super::state::{LobbyPhase, MultiplayerLobby};

/// Guest dialling the host (Steam SDR or iroh QUIC). Generous: covers relay
/// selection and NAT traversal on a slow link.
const DIALLING_BUDGET: f32 = 30.0;
/// Host that has seen the friend enter the lobby but never got an SDR connection.
const HOST_AWAITING_SOCKET_BUDGET: f32 = 30.0;
/// `create_lobby` dispatched, FnOnce result never came back.
const LOBBY_CREATE_BUDGET: f32 = 15.0;
/// `join_lobby` dispatched, FnOnce result never came back. Bounds a Steam
/// call-result round trip, so it is the tightest budget here — keep it generous.
const LOBBY_JOIN_BUDGET: f32 = 25.0;
/// Transport is up but the peer's `PlayerInfo` never arrived. That message is
/// sent exactly once with no retry and no ack, so a dropped one strands both
/// peers permanently.
const HANDSHAKE_BUDGET: f32 = 20.0;

/// Coarse identity of a phase, for change detection. `LobbyPhase` itself carries
/// payloads (wizard lists, failure reasons) that churn without representing a
/// change in what we are waiting for.
// `pub(super)` only because both appear in `timeout_stalled_lobby_phase`'s
// `Local<>` params, which makes them part of the system's signature.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) enum PhaseKind {
    Connect,
    Hosting,
    Joining,
    SteamHosting,
    SteamJoining,
    Handshake,
    WizardSelect,
    Failed,
}

impl PhaseKind {
    fn of(phase: &LobbyPhase) -> Self {
        match phase {
            LobbyPhase::Connect => Self::Connect,
            LobbyPhase::Hosting => Self::Hosting,
            LobbyPhase::Joining => Self::Joining,
            LobbyPhase::SteamHosting => Self::SteamHosting,
            LobbyPhase::SteamJoining => Self::SteamJoining,
            LobbyPhase::Handshake => Self::Handshake,
            LobbyPhase::WizardSelect { .. } => Self::WizardSelect,
            LobbyPhase::Failed { .. } => Self::Failed,
        }
    }
}

/// Same idea for the Steam lobby state — we care which stage it is in, not which
/// lobby id it holds.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) enum SteamKind {
    None,
    Idle,
    Creating,
    Hosting,
    AwaitingJoin,
    Joined,
}

impl SteamKind {
    fn of(state: Option<&SteamLobbyState>) -> Self {
        match state {
            None => Self::None,
            Some(SteamLobbyState::Idle) => Self::Idle,
            Some(SteamLobbyState::Creating) => Self::Creating,
            Some(SteamLobbyState::Hosting { .. }) => Self::Hosting,
            Some(SteamLobbyState::AwaitingJoin { .. }) => Self::AwaitingJoin,
            Some(SteamLobbyState::Joined { .. }) => Self::Joined,
        }
    }
}

/// How long this `(phase, steam lobby stage)` pair may sit without progress.
/// `None` means "wait forever, legitimately".
///
/// The Steam stage matters as much as the phase: `SteamHosting` + `Hosting` is a
/// host waiting on a *human* to accept an invite and must never expire, while
/// `SteamHosting` + `Joined` means the friend is already in the lobby and only the
/// SDR handshake is outstanding — that must expire.
fn budget_for(phase: PhaseKind, steam: SteamKind) -> Option<f32> {
    // Pending Steam call results are bounded regardless of the visible phase:
    // these are the two states where a dropped callback stalls forever.
    match steam {
        SteamKind::Creating => return Some(LOBBY_CREATE_BUDGET),
        SteamKind::AwaitingJoin => return Some(LOBBY_JOIN_BUDGET),
        _ => {}
    }

    match phase {
        PhaseKind::SteamJoining | PhaseKind::Joining => Some(DIALLING_BUDGET),
        PhaseKind::SteamHosting if steam == SteamKind::Joined => Some(HOST_AWAITING_SOCKET_BUDGET),
        PhaseKind::Handshake => Some(HANDSHAKE_BUDGET),
        // Waiting on a human (an invite to be accepted, a code to be pasted), or
        // not waiting on anything at all.
        PhaseKind::SteamHosting
        | PhaseKind::Hosting
        | PhaseKind::Connect
        | PhaseKind::WizardSelect
        | PhaseKind::Failed => None,
    }
}

/// Player-facing explanation for each expiry, phrased as something actionable.
fn timeout_reason(phase: PhaseKind, steam: SteamKind) -> &'static str {
    match steam {
        SteamKind::Creating => {
            "Steam didn't finish creating the lobby. Check that Steam is online and try again."
        }
        SteamKind::AwaitingJoin => {
            "Steam didn't respond to the join request. Your friend's lobby may have closed — ask them to invite you again."
        }
        _ => match phase {
            PhaseKind::SteamJoining => {
                "Couldn't reach your friend. They may have closed the game or left the lobby — ask them to invite you again."
            }
            PhaseKind::Joining => {
                "Couldn't reach the host. Check the connection code and that they're still hosting."
            }
            PhaseKind::SteamHosting => {
                "Your friend joined the lobby but the connection never completed. Ask them to try again."
            }
            PhaseKind::Handshake => {
                "Connected, but your opponent never finished setting up. Try connecting again."
            }
            _ => "The connection timed out.",
        },
    }
}

/// Fails a lobby phase that has sat too long without progress.
///
/// Deliberately does NOT reset to baseline: the player needs to read the reason
/// first. The Failed panel's Try Again button is what runs the reset.
pub(super) fn timeout_stalled_lobby_phase(
    time: Res<Time>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut connection: ResMut<NetworkConnection>,
    steam_lobby: Option<Res<SteamLobbyState>>,
    mut elapsed: Local<f32>,
    mut last_seen: Local<Option<(PhaseKind, SteamKind)>>,
) {
    let phase = PhaseKind::of(&lobby.phase);
    let steam = SteamKind::of(steam_lobby.as_deref());
    let current = (phase, steam);

    // Any movement in either state machine counts as progress — restart the clock.
    if *last_seen != Some(current) {
        *last_seen = Some(current);
        *elapsed = 0.0;
        return;
    }

    let Some(budget) = budget_for(phase, steam) else {
        return;
    };

    *elapsed += time.delta_secs();
    if *elapsed < budget {
        return;
    }

    let reason = timeout_reason(phase, steam);
    warn!(
        "[MP Lobby] Connect stalled for {budget:.0}s — failing. phase={:?} state={:?} mode={:?} steam_lobby={:?}",
        lobby.phase,
        connection.state,
        connection.mode,
        steam_lobby.as_deref(),
    );
    connection.error = Some(reason.to_string());
    // Mark the CONNECTION failed too, not just the phase.
    //
    // `sync_lobby_with_connection` treats a live `Connecting`/`WaitingForSignaling`
    // under a Failed panel as "a new attempt started underneath, resume it" — which
    // is right for a fresh invite. But if we left the stalled attempt's state
    // untouched, that rule would bounce us straight back to `SteamJoining` on the
    // next frame, re-arm this timer, and oscillate every `budget` seconds without
    // the player ever getting a stable error to act on. A timeout IS a connection
    // failure; say so.
    //
    // This still leaves the door open to a very late success: if the socket does
    // eventually reach `Connected`, the resume rule fires and the lobby continues
    // into the handshake rather than stranding on Failed.
    connection.state = ConnectionState::Failed;
    lobby.phase = LobbyPhase::Failed {
        reason: reason.to_string(),
    };
    lobby.status_message = None;
    *elapsed = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_waiting_on_a_human_never_expires() {
        // Invite sent, friend hasn't accepted yet — this is a person, not a stall.
        assert_eq!(
            budget_for(PhaseKind::SteamHosting, SteamKind::Hosting),
            None
        );
        // Code-share host waiting for someone to paste the code.
        assert_eq!(budget_for(PhaseKind::Hosting, SteamKind::None), None);
        // Idle screens.
        assert_eq!(budget_for(PhaseKind::Connect, SteamKind::Idle), None);
        assert_eq!(budget_for(PhaseKind::WizardSelect, SteamKind::Joined), None);
        // Already failed — don't re-fail it.
        assert_eq!(budget_for(PhaseKind::Failed, SteamKind::Idle), None);
    }

    #[test]
    fn a_host_whose_friend_already_joined_does_expire() {
        // The friend is in the lobby; only the SDR handshake is outstanding.
        assert_eq!(
            budget_for(PhaseKind::SteamHosting, SteamKind::Joined),
            Some(HOST_AWAITING_SOCKET_BUDGET)
        );
    }

    #[test]
    fn pending_steam_callbacks_expire_regardless_of_phase() {
        // These stall forever today if the FnOnce result never arrives, and the
        // visible phase can be anything (a Steam invite accepted from the Connect
        // screen never leaves `Connect` until `join_lobby` reports back).
        for phase in [
            PhaseKind::Connect,
            PhaseKind::SteamHosting,
            PhaseKind::SteamJoining,
        ] {
            assert_eq!(
                budget_for(phase, SteamKind::Creating),
                Some(LOBBY_CREATE_BUDGET)
            );
            assert_eq!(
                budget_for(phase, SteamKind::AwaitingJoin),
                Some(LOBBY_JOIN_BUDGET)
            );
        }
    }

    #[test]
    fn dialling_and_handshake_expire() {
        assert_eq!(
            budget_for(PhaseKind::SteamJoining, SteamKind::Joined),
            Some(DIALLING_BUDGET)
        );
        assert_eq!(
            budget_for(PhaseKind::Joining, SteamKind::None),
            Some(DIALLING_BUDGET)
        );
        assert_eq!(
            budget_for(PhaseKind::Handshake, SteamKind::Joined),
            Some(HANDSHAKE_BUDGET)
        );
    }
}
