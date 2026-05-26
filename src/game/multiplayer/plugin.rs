//! Multiplayer game plugin.
//!
//! Registers only multiplayer-specific systems. The host reuses all single-player
//! gameplay systems (movement, combat, spells, etc.) via `is_gameplay_running`,
//! so this plugin only adds networking, guest rendering, score screen, and
//! disconnect detection.

use bevy::prelude::*;

use crate::game::plugin::PostCombatSet;
use crate::game::units::wizard::systems::cancel_active_casts;
use crate::networking::session::{is_multiplayer_guest, is_multiplayer_host};
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;

use super::crdt_sync;
use super::guest_systems;
use super::host_systems;
use super::loading;
use super::spell_sync;
use super::systems::{
    cleanup_mp_disconnected, cleanup_mp_game, cleanup_mp_pause_menu, cleanup_mp_score_screen,
    detect_mp_disconnect, detect_mp_loading_disconnect, handle_mp_disconnected_buttons,
    handle_mp_pause_buttons, handle_mp_score_buttons, handle_mp_score_messages, in_mp_disconnected,
    in_mp_paused, in_mp_running, in_mp_score_screen, init_mp_game, mp_escape_key_handler,
    setup_mp_disconnected, setup_mp_pause_menu, setup_mp_score_screen,
};

/// Plugin that manages multiplayer gameplay.
///
/// The host reuses all single-player gameplay systems (registered by `GamePlugin`,
/// `UnitsPlugin`, `InfantryPlugin`, `ArcherPlugin`, `KingPlugin`, etc.) via the
/// `is_gameplay_running` run condition. This plugin only adds:
/// - Loading and camera setup
/// - Resource init/cleanup
/// - Host networking (ID assignment, snapshots, king death check)
/// - Guest snapshot rendering and game-over handling
/// - Score screen UI and rematch flow
/// - Disconnect detection
pub struct MultiplayerGamePlugin;

impl Plugin for MultiplayerGamePlugin {
    fn build(&self, app: &mut App) {
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
        app.add_systems(OnEnter(AppState::MultiplayerGame), init_mp_game);
        app.add_systems(OnExit(AppState::MultiplayerGame), cleanup_mp_game);

        // ── Cancel casts on exit Running ─────────────────────────────
        // Reuses the SP cancel_active_casts system so both wizards'
        // CastingState gets reset when transitioning to ScoreScreen/Paused.
        app.add_systems(OnExit(MultiplayerGameState::Running), cancel_active_casts);

        // ── CRDT Health Sync (both host and guest) ──────────────────
        // attach_crdt_health: adds CrdtHealth to new entities with Health
        // sync_health_to_crdt: detects local damage/healing, writes to CRDT,
        //   re-derives Health from converged CRDT state
        let mp_running = in_mp_running;
        app.add_systems(Update, crdt_sync::attach_crdt_health.run_if(mp_running));
        app.add_systems(
            Update,
            crdt_sync::sync_health_to_crdt
                .after(PostCombatSet)
                .run_if(mp_running),
        );
        app.add_systems(Update, crdt_sync::receive_wall_placement.run_if(mp_running));

        // ── Host: MP King Death Check ────────────────────────────────
        // Replaces SP's check_win_lose_conditions during multiplayer.
        // Runs after PostCombatSet so corpses have been created.
        let mp_host = in_mp_running.and(is_multiplayer_host);

        app.add_systems(
            Update,
            host_systems::check_mp_king_death
                .after(PostCombatSet)
                .run_if(mp_host.clone()),
        );

        // ── Host Networking: ID Assignment + Unit Snapshots ──────────
        app.add_systems(
            Update,
            (
                host_systems::assign_network_ids,
                host_systems::send_state_snapshots,
            )
                .chain()
                .after(PostCombatSet)
                .run_if(mp_host),
        );

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
                .run_if(mp_running),
        );

        app.add_systems(
            Update,
            (
                spell_sync::receive_spell_visual_snapshot,
                spell_sync::apply_remote_spell_snapshot,
            )
                .chain()
                .run_if(mp_running),
        );

        // ── Guest: Unit Snapshot Rendering + CRDT Send ─────────────────
        // `apply_state_snapshot` writes ghost `Velocity` straight from the
        // host's `UnitSnapshot.vx/vz` fields. Tagging it with
        // `GuestSnapshotSet` lets the shared animation systems (registered
        // in `UnitsPlugin`) order themselves after the write via
        // `.after(GuestSnapshotSet)` so they always see the host's
        // authoritative velocity for this frame.
        app.add_systems(
            Update,
            (
                guest_systems::apply_state_snapshot
                    .in_set(crate::game::units::GuestSnapshotSet),
                guest_systems::send_crdt_snapshot,
            )
                .chain()
                .run_if(mp_running.and(is_multiplayer_guest)),
        );

        // ── Host: Receive Guest CRDT Updates ──────────────────────────
        // Must run after PostCombatSet (so host damage is applied first) but
        // before sync_health_to_crdt (so the merged CRDT is visible when
        // sync re-derives Health.current).
        app.add_systems(
            Update,
            host_systems::receive_crdt_snapshot
                .after(PostCombatSet)
                .before(crdt_sync::sync_health_to_crdt)
                .run_if(mp_running.and(is_multiplayer_host)),
        );

        // ── Host: Receive Guest Teleport Messages ────────────────────
        app.add_systems(
            Update,
            host_systems::receive_teleport_message.run_if(mp_running.and(is_multiplayer_host)),
        );

        // ── Host: Receive Guest Spell Hits ────────────────────────────
        // Ordered explicitly `.before(process_pending_damage_effects)` so
        // the inserted `PendingDamageEffect` is at least visible to the
        // pending-processor on the very next frame after the command flush;
        // without the ordering Bevy could schedule the processor first and
        // delay the DoT by two frames instead of one.
        app.add_systems(
            Update,
            host_systems::receive_spell_hit_messages
                .before(crate::game::units::systems::process_pending_damage_effects)
                .run_if(mp_running.and(is_multiplayer_host)),
        );

        // ── Guest: Forward Local Spell Hits to Host ───────────────────
        // When a guest's local spell explosion damages a ghost unit (which
        // inserts `PendingDamageEffect` via SP's `apply_spell_damage`), this
        // system catches the new component, ships a `SpellHitUnit` message
        // to the host, then removes the local `PendingDamageEffect` so the
        // guest doesn't double-tick the DoT against the host-ticked one.
        app.add_systems(
            Update,
            guest_systems::forward_spell_hits_to_host
                .run_if(mp_running.and(is_multiplayer_guest)),
        );

        // ── Guest: Game Over Message ──────────────────────────────────
        app.add_systems(
            Update,
            guest_systems::handle_game_over_message.run_if(mp_running.and(is_multiplayer_guest)),
        );

        // ── Escape Key (Running → Paused toggle) ──────────────────────
        app.add_systems(
            Update,
            mp_escape_key_handler
                .run_if(in_state(AppState::MultiplayerGame).and(in_mp_running.or(in_mp_paused))),
        );

        // ── Escape Menu (Paused overlay) ──────────────────────────────
        app.add_systems(OnEnter(MultiplayerGameState::Paused), setup_mp_pause_menu);
        app.add_systems(OnExit(MultiplayerGameState::Paused), cleanup_mp_pause_menu);
        app.add_systems(
            Update,
            handle_mp_pause_buttons
                .in_set(ButtonActionSet)
                .run_if(in_mp_paused),
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
            setup_mp_score_screen,
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
    }
}
