//! Span A: multiplayer loading, camera, and resource init/cleanup.

use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use crate::game::multiplayer::loading;
use crate::game::multiplayer::pause_request::{self, RequestGamePauseMessage};
use crate::game::multiplayer::systems::{
    cleanup_mp_game, init_mp_game, sync_wizard_type_from_session,
};
use crate::state::AppState;

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    // ── Auto-pause request bus ───────────────────────────────────
    // A single message + consumer that turns any "input was taken away" trigger
    // (Steam overlay, controller unplug, focus loss) into the correct pause for
    // the current mode (single-player + co-op sync freeze; versus + Urgent co-op
    // pop a local, non-freezing pause menu mirroring manual Escape). Registered
    // here in Span A because the consumer is cross-cutting — it handles
    // single-player too, not just co-op. `MultiplayerGamePlugin` is added
    // unconditionally, so this exists in every mode. Gated ONLY on the message
    // (not `is_gameplay_active`, which would exclude the co-op guest); the
    // consumer self-gates on the `Option` state resources.
    app.add_message::<RequestGamePauseMessage>();
    app.add_systems(
        Update,
        pause_request::apply_pause_requests.run_if(on_message::<RequestGamePauseMessage>),
    );

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
