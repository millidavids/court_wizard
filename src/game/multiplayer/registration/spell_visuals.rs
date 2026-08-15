//! Span D: bidirectional spell-visual sync (collect/send + receive/apply) and
//! Excremage ghost theming + pending-cast-event lifecycle.

use bevy::prelude::*;

use crate::game::multiplayer::systems::in_mp_running;
use crate::game::multiplayer::{excremage_theming, spell_sync};
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::networking::session::is_multiplayer_host;
use crate::state::AppState;

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    let both_peers = in_mp_running.or(is_gameplay_running.and(is_multiplayer_host));

    // ── Bidirectional Spell Visual Sync ──────────────────────────
    // Both host and guest collect local spell visuals and send them,
    // then receive and render the other player's spells as ghosts.
    app.add_systems(
        Update,
        (
            spell_sync::collect_spell_effect_snapshots,
            spell_sync::collect_spell_projectile_snapshots,
            spell_sync::send_spell_visual_snapshot,
        )
            .chain()
            .after(PostCombatSet)
            .run_if(both_peers.clone()),
    );

    app.add_systems(
        Update,
        (
            spell_sync::receive_spell_visual_snapshot,
            spell_sync::apply_remote_spell_snapshot,
            spell_sync::apply_remote_cast_events,
            // Regenerate disintegrate beam-tip VFX for the opposing client's
            // ghost beam (runs on both peers; the ghost has no DisintegrateBeam).
            spell_sync::spawn_ghost_beam_impact_vfx,
            // Brown the opponent's spell ghosts when they're an Excremage
            // (per-entity material clone — never touches our own spells).
            excremage_theming::theme_remote_excremage_ghosts
                .run_if(excremage_theming::is_remote_excremage),
        )
            .chain()
            .run_if(both_peers.clone()),
    );
    app.init_resource::<excremage_theming::ExcremageGhostMaterials>();
    app.add_systems(
        OnExit(AppState::MultiplayerGame),
        excremage_theming::clear_excremage_ghost_materials,
    );
    // The co-op HOST never enters `AppState::MultiplayerGame` — it plays in
    // `AppState::InGame` — but `theme_remote_excremage_ghosts` runs under
    // `both_peers`, which includes it. Without this mirror the browned-material
    // clones it creates were never freed, so the cache grew for the whole process:
    // every level, every reconnect, until the game was restarted.
    app.add_systems(
        OnExit(AppState::InGame),
        excremage_theming::clear_excremage_ghost_materials,
    );

    // Outgoing one-shot cast VFX events. Casting handlers push to this
    // via `vfx::systems::emit_cast_event` (or the `_synced` wrappers);
    // `send_spell_visual_snapshot` drains it into the snapshot once per
    // tick. Initialized for both peers — guest casts ship events too.
    // `mp_active` is toggled on MP state enter/exit so single-player
    // calls to `emit_cast_event` no-op (the drain only runs in MP).
    app.init_resource::<spell_sync::PendingCastEvents>();
    app.add_systems(
        OnEnter(AppState::MultiplayerGame),
        spell_sync::mark_pending_events_mp_active,
    );
    app.add_systems(
        OnExit(AppState::MultiplayerGame),
        spell_sync::mark_pending_events_mp_inactive,
    );
}
