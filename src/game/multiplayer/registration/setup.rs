//! Span A: multiplayer loading, camera, and resource init/cleanup.

use bevy::prelude::*;

use crate::game::multiplayer::loading;
use crate::game::multiplayer::systems::{
    cleanup_mp_game, init_mp_game, sync_wizard_type_from_session,
};
use crate::state::AppState;

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    // ── Multiplayer Loading ──────────────────────────────────────
    app.add_systems(
        OnEnter(AppState::MultiplayerLoading),
        loading::init_mp_loading,
    )
    .add_systems(
        Update,
        loading::process_mp_spawn_queue.run_if(in_state(AppState::MultiplayerLoading)),
    )
    .add_systems(
        OnExit(AppState::MultiplayerLoading),
        loading::cleanup_mp_loading,
    );

    // ── Camera ───────────────────────────────────────────────────
    app.add_systems(OnEnter(AppState::MultiplayerGame), loading::setup_mp_camera);
    app.add_systems(OnExit(AppState::MultiplayerGame), loading::restore_camera);

    // ── Resource Init / Cleanup ──────────────────────────────────
    // Sync the local wizard type from the session FIRST, so every archetype
    // run-condition (`is_warglock`, `is_meteorologist`, …) and the systems
    // gated on them evaluate against the wizard actually being played. Other
    // archetype-gated `OnEnter(MultiplayerGame)` systems are ordered
    // `.after(systems::sync_wizard_type_from_session)`.
    app.add_systems(
        OnEnter(AppState::MultiplayerGame),
        (
            sync_wizard_type_from_session,
            init_mp_game,
            // Excremage recolors the shared spell materials brown. The SP path
            // only does this on `OnEnter(AppState::Loading)`, which never fires
            // for an MP match — so re-run it here, AFTER the wizard type is
            // synced. The system is idempotent and self-restoring (it skips a
            // no-op and un-browns when the type isn't Excremage), so it's safe
            // ungated and also handles a non-Excremage MP match after an
            // Excremage one.
            crate::game::units::wizard::spells::visual_assets::refresh_spell_visuals_for_wizard,
        )
            .chain(),
    );
    app.add_systems(OnExit(AppState::MultiplayerGame), cleanup_mp_game);
}
