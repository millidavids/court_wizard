//! Multiplayer game systems (init, cleanup, score screen, pause, disconnect).

mod disconnected;
mod invite_abandon;
mod lifecycle;
mod pause_menu;
mod score_screen_handlers;
mod score_screen_ui;

// ── lifecycle ────────────────────────────────────────────────────────────────
pub(crate) use lifecycle::sync_wizard_type_from_session;
pub(super) use lifecycle::{
    cleanup_mp_game, in_mp_disconnected, in_mp_paused, in_mp_running, in_mp_score_screen,
    init_mp_game,
};

// ── score_screen_ui ──────────────────────────────────────────────────────────
pub(super) use score_screen_ui::{
    cleanup_mp_score_screen, setup_mp_score_screen, update_mp_stat_values,
};

// ── score_screen_handlers ────────────────────────────────────────────────────
pub(super) use score_screen_handlers::{
    handle_mp_score_buttons, handle_mp_score_messages, mp_score_escape_handler,
};

// ── pause_menu ───────────────────────────────────────────────────────────────
pub(super) use pause_menu::{
    cleanup_mp_pause_menu, handle_mp_forfeit_confirm, handle_mp_pause_buttons,
    mp_escape_key_handler, relabel_mp_resume_for_coop, setup_mp_pause_menu,
};

// ── disconnected ─────────────────────────────────────────────────────────────
pub(super) use disconnected::{
    cleanup_mp_disconnected, detect_mp_disconnect, detect_mp_loading_disconnect,
    handle_mp_disconnected_buttons, setup_mp_disconnected,
};

// ── invite_abandon ───────────────────────────────────────────────────────────
pub(super) use invite_abandon::abandon_run_for_steam_invite;
