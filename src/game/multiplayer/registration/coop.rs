//! Span B: co-op host lifecycle, synchronized co-op pause, and cast-cancel on exit.

use bevy::prelude::*;

use crate::game::multiplayer::{coop, coop_pause};
use crate::game::run_conditions::{is_gameplay_active, is_gameplay_running};
use crate::game::units::wizard::systems::cancel_active_casts;
use crate::networking::session::{is_coop_session, is_multiplayer_guest, is_multiplayer_host};
use crate::state::{AppState, InGameState, MultiplayerGameState};

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    // ── Co-op Host (runs in AppState::InGame, not MultiplayerGame) ───────
    // The co-op host plays a native single-player endless/roguelite game, so
    // it never enters MultiplayerGame and never runs `init_mp_game`. Stand up
    // its networking resources on InGame enter, tear them down on exit. Each
    // co-op level is an independent match (endless/roguelite loop via the
    // tower); the CONNECTION is preserved across levels (the lobby reset skips
    // co-op), so the host re-launches the next level from the tower. All
    // self-gate on the co-op-host session (no-op in SP and for the guest).
    app.init_resource::<coop::CoopPeerInfo>();
    app.add_systems(OnEnter(AppState::InGame), coop::init_coop_host);
    app.add_systems(OnExit(AppState::InGame), coop::cleanup_coop_host);
    // +30% attacker effectiveness while a co-op guest is connected (cleared
    // the instant they disconnect). Co-op host only.
    app.add_systems(
        Update,
        coop::apply_coop_difficulty_buff.run_if(is_gameplay_running.and(coop::is_coop_host)),
    );
    // Graceful disconnect: if the guest drops, the host keeps playing solo
    // and the +30% buff lifts (reconnect is a follow-up). Gated on
    // `is_gameplay_active` (NOT `is_gameplay_running`) so it still fires while
    // the host is in a sync-pause (`InGameState::Paused`, where the simulation
    // is frozen) — otherwise a guest dropping mid-pause would strand the host
    // paused forever, since the auto-resume lives in this system.
    app.add_systems(
        Update,
        coop::detect_coop_guest_disconnect.run_if(is_gameplay_active.and(coop::is_coop_host)),
    );
    // Co-op level-end coordination: the host announces each level's result at
    // its score screen; the in-place guest waits for the next level (or the
    // run-ended message) via `receive_coop_lifecycle`.
    app.add_systems(
        OnEnter(InGameState::ScoreScreen),
        coop::send_coop_level_over.run_if(coop::is_coop_host),
    );
    app.add_systems(
        Update,
        coop::receive_coop_lifecycle.run_if(
            in_state(AppState::MultiplayerGame)
                .and(is_coop_session)
                .and(is_multiplayer_guest),
        ),
    );

    // ── Synchronized Co-op Pause ─────────────────────────────────
    // Either peer pausing pauses both; only the initiator resumes; the
    // initiator heartbeats the authoritative state so a dropped packet can't
    // strand the other player. All gated on a co-op, non-Urgent session.
    app.init_resource::<coop_pause::CoopPauseState>();
    app.add_systems(
        OnEnter(AppState::InGame),
        coop_pause::reset_coop_pause_state.run_if(coop::is_coop_host),
    );
    app.add_systems(
        OnEnter(AppState::MultiplayerGame),
        coop_pause::reset_coop_pause_state.run_if(is_multiplayer_guest),
    );
    // Local pause input (both peers — branches on role internally).
    app.add_systems(
        Update,
        coop_pause::coop_pause_input.run_if(coop_pause::coop_sync_pause_enabled),
    );
    // Host applies incoming pause state to InGameState.
    app.add_systems(
        Update,
        coop_pause::coop_pause_receive_host.run_if(
            in_state(AppState::InGame)
                .and(coop_pause::coop_sync_pause_enabled)
                .and(is_multiplayer_host),
        ),
    );
    // Guest applies incoming pause state to MultiplayerGameState.
    app.add_systems(
        Update,
        coop_pause::coop_pause_receive_guest.run_if(
            in_state(AppState::MultiplayerGame)
                .and(coop_pause::coop_sync_pause_enabled)
                .and(is_multiplayer_guest),
        ),
    );
    // Initiator re-broadcasts the authoritative state (self-healing).
    app.add_systems(
        Update,
        coop_pause::coop_pause_heartbeat.run_if(coop_pause::coop_sync_pause_enabled),
    );

    // ── Cancel casts on exit Running ─────────────────────────────
    // Reuses the SP cancel_active_casts system so both wizards'
    // CastingState gets reset when transitioning to ScoreScreen/Paused.
    app.add_systems(OnExit(MultiplayerGameState::Running), cancel_active_casts);
}
