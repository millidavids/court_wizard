//! Span E: guest snapshot rendering + the host/guest message-relay systems
//! (CRDT, teleport, spell hits, status effects, raise-corpse, dispel, game-over).

use bevy::prelude::*;

use crate::game::multiplayer::systems::in_mp_running;
use crate::game::multiplayer::{
    crdt_sync, excremage_theming, guest_systems, host_systems, score_stats, spell_sync,
};
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::networking::session::{is_multiplayer_guest, is_multiplayer_host};

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    let mp_running = in_mp_running;
    let host_net = is_gameplay_running.and_then(is_multiplayer_host);

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
            guest_systems::apply_state_snapshot.in_set(crate::game::units::GuestSnapshotSet),
            guest_systems::send_crdt_snapshot,
        )
            .chain()
            // `apply_state_snapshot` shares `ResMut<Assets<StandardMaterial>>`
            // with the ghost-spawning spell-sync systems above. Order it after
            // them so the material-pool writes have a deterministic order (no
            // schedule ambiguity) on the guest. `apply_remote_cast_events` runs
            // after `apply_remote_spell_snapshot` in its own chain, so this
            // covers that one transitively; the Excremage theming pass is the
            // only other StandardMaterial writer and is named explicitly.
            .after(spell_sync::apply_remote_cast_events)
            .after(excremage_theming::theme_remote_excremage_ghosts)
            .run_if(mp_running.and_then(is_multiplayer_guest)),
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
            .run_if(host_net.clone()),
    );

    // ── Host: Receive Guest Teleport Messages ────────────────────
    app.add_systems(
        Update,
        host_systems::receive_teleport_message.run_if(host_net.clone()),
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
            .run_if(host_net.clone()),
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
            .before(crdt_sync::sync_health_to_crdt)
            .run_if(mp_running.and_then(is_multiplayer_guest)),
    );

    // ── Guest: Forward Status Effects to Host ─────────────────────
    // Generic forwarder watching for status components newly inserted
    // on ghost units, so guest-cast Sleep/Root/Mark/Haste/etc. take
    // hold on host-authoritative units. See `forward_status_effects_to_host`.
    app.add_systems(
        Update,
        guest_systems::forward_status_effects_to_host
            .run_if(mp_running.and_then(is_multiplayer_guest)),
    );

    // Pair each forwarded marker with a cleanup that removes the marker
    // the frame after its underlying status component is removed. Without
    // this, a second cast of the same status on the same ghost would be
    // silently dropped (the Without<StatusEffectForwarded<T>> filter
    // would keep the stale marker forever).
    use crate::game::units::components as comp;
    use crate::game::units::status_effects as sfx;
    use guest_systems::cleanup_forwarded_marker as cleanup;
    app.add_systems(
        Update,
        (
            cleanup::<sfx::SleepModifier>,
            cleanup::<sfx::RootedModifier>,
            cleanup::<sfx::PolymorphedModifier>,
            cleanup::<comp::MindControlled>,
            cleanup::<sfx::BanishedModifier>,
            cleanup::<sfx::MarkedForDeathModifier>,
            cleanup::<sfx::HasteModifier>,
            cleanup::<sfx::BattleHymnModifier>,
            cleanup::<sfx::BerserkerRageModifier>,
            cleanup::<comp::TemporaryHitPoints>,
            cleanup::<comp::SlowMovementModifier>,
            cleanup::<sfx::Stunned>,
            cleanup::<sfx::FogEvasionModifier>,
            // No Knockback cleanup: the knockback arm of
            // `forward_status_effects_to_host` removes the ghost's Knockback
            // immediately after forwarding instead of tagging it, so no
            // `StatusEffectForwarded<Knockback>` marker is ever created.
        )
            .run_if(mp_running.and_then(is_multiplayer_guest)),
    );

    // ── Host: Receive Generic Status Effects ─────────────────────
    app.add_systems(
        Update,
        host_systems::receive_apply_status_effect.run_if(host_net.clone()),
    );

    // ── Host: Receive Raise-Corpse Messages ─────────────────────
    app.add_systems(
        Update,
        host_systems::receive_raise_corpse_messages.run_if(host_net.clone()),
    );

    // ── Both peers: Receive Dispel Messages ───────────────────────
    // Each peer's own dispel impact only despawns effects it owns, so the
    // remote peer's are reachable only through this hand-off. Registering it
    // host-only meant a host-cast dispel simply passed through everything the
    // guest had summoned.
    app.add_systems(
        Update,
        host_systems::receive_dispel_messages.run_if(mp_running.or_else(host_net.clone())),
    );

    // ── Guest: Game Over Message ──────────────────────────────────
    app.add_systems(
        Update,
        guest_systems::handle_game_over_message
            // Run after the tally accumulator so the guest's own final spell
            // stats are folded in before it builds MatchStats / sends its report.
            .after(score_stats::accumulate_wizard_spell_stats)
            .run_if(mp_running.and_then(is_multiplayer_guest)),
    );
}
