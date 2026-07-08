//! Cross-cutting "request a pause" message and its single mode-aware consumer.
//!
//! Several triggers want to pause the game when input is taken away — the Steam
//! overlay opening (`steam::overlay_pause`), the active controller disconnecting
//! (`gamepad::systems::connection`), and the window losing focus
//! (`shared_systems::auto_pause_on_focus_loss`). Rather than duplicate the
//! mode-specific pause logic (and couple those modules to the networking layer),
//! each trigger just writes [`RequestGamePauseMessage`], and this one consumer
//! decides what pausing actually means:
//!
//! - **Single-player:** transition `InGameState::Running → Paused`.
//! - **Co-op (synchronized):** initiate the shared `CoopPauseSync` handshake via
//!   [`coop_pause::do_initiate_pause`] so both peers pause and only the initiator
//!   can resume.
//! - **Versus / Urgent co-op:** set the local peer's own `Paused` menu, mirroring
//!   manual Escape. That state is non-freezing in these modes (the sim keeps
//!   running — see `run_conditions::is_gameplay_running`), so a real-time P2P match
//!   is never frozen; the pause is local-only and sends no network message.

use bevy::prelude::*;

use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::{InGameState, MultiplayerGameState};

use super::coop_pause::{self, CoopPauseState, LocalState};

/// Broadcast request to pause the game, written by any auto-pause trigger.
#[derive(Message)]
pub(crate) struct RequestGamePauseMessage;

/// Applies a pause request for the current game mode. Gated by
/// `run_if(on_message::<RequestGamePauseMessage>)`; the `Option` parameters make
/// it a safe no-op outside gameplay (in menus the state resources are absent).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_pause_requests(
    mut events: MessageReader<RequestGamePauseMessage>,
    session: Option<Res<MultiplayerSession>>,
    pause: Option<ResMut<CoopPauseState>>,
    connection: Option<ResMut<NetworkConnection>>,
    sp_state: Option<Res<State<InGameState>>>,
    mut next_sp: Option<ResMut<NextState<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_mp: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    // Fully drain so a duplicate request (e.g. overlay-open AND focus-loss firing
    // the same frame) can't re-fire next frame off a leftover buffered message.
    if events.read().count() == 0 {
        return;
    }

    // ── Co-op (synchronized): initiate the shared pause handshake ─────
    if let Some(session) = session.as_deref()
        && session.coop_pause_synced()
    {
        let (Some(mut pause), Some(mut connection)) = (pause, connection) else {
            return;
        };
        if pause.active {
            return; // already paused
        }
        let is_host = session.role == PeerRole::Host;
        let running = if is_host {
            matches!(
                coop_pause::local_state_host(sp_state.as_deref()),
                LocalState::Running
            )
        } else {
            matches!(
                coop_pause::local_state_guest(mp_state.as_deref()),
                LocalState::Running
            )
        };
        if !running {
            return;
        }
        coop_pause::do_initiate_pause(
            is_host,
            session.role,
            &mut pause,
            &mut connection,
            &mut next_sp,
            &mut next_mp,
        );
        return;
    }

    // ── Single-player, versus, or Urgent co-op ────────────────────────
    // Set the local peer's own mode-appropriate Paused state, mirroring manual
    // Escape. The peer is in exactly ONE gameplay state machine (host/SP →
    // InGameState; versus + all guests → MultiplayerGameState — AppState holds one
    // variant, so they are never both present). In single-player this freezes the
    // sim; in versus and Urgent co-op the Paused state is a menu overlay that KEEPS
    // the sim running (`run_conditions::is_gameplay_running`), so a real-time P2P
    // match is never frozen. Synchronized co-op is handled by the branch above.
    if matches!(
        coop_pause::local_state_host(sp_state.as_deref()),
        LocalState::Running
    ) && let Some(n) = next_sp.as_mut()
    {
        n.set(InGameState::Paused);
    } else if matches!(
        coop_pause::local_state_guest(mp_state.as_deref()),
        LocalState::Running
    ) && let Some(n) = next_mp.as_mut()
    {
        n.set(MultiplayerGameState::Paused);
    }
}
