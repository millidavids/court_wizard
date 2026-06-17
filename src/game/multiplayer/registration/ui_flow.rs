//! Span F: escape-key handling, pause menu, disconnected overlay, score screen,
//! and in-match / loading disconnect detection.

use bevy::prelude::*;

use crate::game::multiplayer::host_systems;
use crate::game::multiplayer::score_stats;
use crate::game::multiplayer::systems::{
    abandon_run_for_steam_invite, cleanup_mp_disconnected, cleanup_mp_pause_menu,
    cleanup_mp_score_screen, detect_mp_disconnect, detect_mp_loading_disconnect,
    handle_mp_disconnected_buttons, handle_mp_forfeit_confirm, handle_mp_pause_buttons,
    handle_mp_score_buttons, handle_mp_score_messages, in_mp_disconnected, in_mp_paused,
    in_mp_running, in_mp_score_screen, mp_escape_key_handler, mp_score_escape_handler,
    relabel_mp_resume_for_coop, setup_mp_disconnected, setup_mp_pause_menu, setup_mp_score_screen,
    update_mp_stat_values,
};
use crate::networking::session::is_multiplayer_host;
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    // ── Escape Key (Running → Paused toggle) ──────────────────────
    app.add_systems(
        Update,
        mp_escape_key_handler
            .run_if(in_state(AppState::MultiplayerGame).and(in_mp_running.or(in_mp_paused))),
    );

    // ── Escape Menu (Paused overlay) ──────────────────────────────
    app.add_systems(
        OnEnter(MultiplayerGameState::Paused),
        // In a co-op sync-pause a NON-initiator guest gets a relabeled,
        // disabled-looking Resume button ("Waiting for other player").
        (setup_mp_pause_menu, relabel_mp_resume_for_coop).chain(),
    );
    app.add_systems(OnExit(MultiplayerGameState::Paused), cleanup_mp_pause_menu);
    app.add_systems(
        Update,
        (handle_mp_pause_buttons, handle_mp_forfeit_confirm)
            .in_set(ButtonActionSet)
            .run_if(in_mp_paused),
    );
    // Host: end the match when the guest forfeits. Runs while Running OR
    // Paused so a paused host still processes the guest's forfeit promptly
    // (otherwise the guest sits in limbo until the host unpauses).
    app.add_systems(
        Update,
        host_systems::receive_mp_forfeit
            .run_if((in_mp_running.or(in_mp_paused)).and(is_multiplayer_host)),
    );

    // ── Disconnected Overlay ──────────────────────────────────────
    app.add_systems(
        OnEnter(MultiplayerGameState::Disconnected),
        setup_mp_disconnected,
    );
    app.add_systems(
        OnExit(MultiplayerGameState::Disconnected),
        cleanup_mp_disconnected,
    );
    app.add_systems(
        Update,
        handle_mp_disconnected_buttons
            .in_set(ButtonActionSet)
            .run_if(in_mp_disconnected),
    );

    // ── Score Screen ──────────────────────────────────────────────
    app.add_systems(
        OnEnter(MultiplayerGameState::ScoreScreen),
        (
            setup_mp_score_screen,
            // Mirror single-player (which stops SFX on InGameState::ScoreScreen):
            // silence lingering spell / looping / ambience sounds the instant
            // the match ends so they don't drone under the scoreboard.
            crate::game::shared_systems::stop_all_sfx,
        ),
    );
    app.add_systems(
        OnExit(MultiplayerGameState::ScoreScreen),
        cleanup_mp_score_screen,
    );
    app.add_systems(
        Update,
        (
            handle_mp_score_buttons.in_set(ButtonActionSet),
            handle_mp_score_messages,
            // Escape on the score screen disconnects to the main menu (same as
            // the Disconnect button). Keyboard handler — not in ButtonActionSet.
            mp_score_escape_handler,
            // Reactively refresh stat values when MatchStats changes (e.g.
            // the host's enemy column fills in from the guest's report).
            update_mp_stat_values.run_if(
                resource_exists::<score_stats::MatchStats>
                    .and(resource_changed::<score_stats::MatchStats>),
            ),
        )
            .run_if(in_mp_score_screen),
    );

    // ── Disconnect Detection ──────────────────────────────────────
    // Covers the in-match path (`MultiplayerGame`) and the loading
    // path (`MultiplayerLoading`). The lobby path inside the wizard
    // tower has its own connection-state watcher in
    // `multiplayer_tab::sync::sync_lobby_with_connection`.
    app.add_systems(
        Update,
        detect_mp_disconnect.run_if(in_state(AppState::MultiplayerGame)),
    );
    app.add_systems(
        Update,
        detect_mp_loading_disconnect.run_if(in_state(AppState::MultiplayerLoading)),
    );

    // ── Accept-invite-from-anywhere: abandon an active run ────────
    // When a Steam invite is accepted mid-run, tear the player's own match down
    // to the main menu so the steam-side `route_pending_steam_join` can then route
    // into the multiplayer tab and connect. Gated on the intent existing AND being
    // in an active run; the system body branches InGame vs MultiplayerGame.
    app.add_systems(
        Update,
        abandon_run_for_steam_invite.run_if(
            resource_exists::<crate::steam::multiplayer::PendingSteamJoin>
                .and(in_state(AppState::InGame).or(in_state(AppState::MultiplayerGame))),
        ),
    );
}
