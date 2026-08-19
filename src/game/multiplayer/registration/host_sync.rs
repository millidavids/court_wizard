//! Span C: CRDT health sync, wizard spell-stat tally, host king-death check,
//! and host ID-assignment / unit-snapshot networking.

use bevy::prelude::*;

use crate::game::multiplayer::systems::in_mp_running;
use crate::game::multiplayer::{crdt_sync, host_systems, score_stats};
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::is_gameplay_running;
use crate::networking::session::is_multiplayer_host;

pub(in crate::game::multiplayer) fn register(app: &mut App) {
    // ── CRDT Health Sync (both host and guest) ──────────────────
    // attach_crdt_health: adds CrdtHealth to new entities with Health
    // sync_health_to_crdt: detects local damage/healing, writes to CRDT,
    //   re-derives Health from converged CRDT state
    // Run-condition vocabulary for the asymmetric co-op topology:
    // - `mp_running`  : MultiplayerGame running (versus host+guest, co-op guest)
    // - `both_peers`  : the above OR the co-op host in `AppState::InGame`
    // - `host_net`    : EITHER host's networking (versus MultiplayerGame OR
    //                   co-op InGame); excludes SP via `is_multiplayer_host`.
    // Match-LIFECYCLE host systems (`check_mp_king_death`, `receive_mp_forfeit`)
    // stay `in_mp_running`-gated — they push `MultiplayerGameState`, which the
    // co-op host (in InGame) doesn't have. Co-op lifecycle is the SP path.
    let both_peers = in_mp_running.or_else(is_gameplay_running.and_then(is_multiplayer_host));
    let host_net = is_gameplay_running.and_then(is_multiplayer_host);
    app.add_systems(
        Update,
        crdt_sync::attach_crdt_health.run_if(both_peers.clone()),
    );
    // `sync_health_to_crdt` records the local Health delta (e.g. damage
    // a guest-cast spell just dealt to a ghost) into the local CRDT
    // slot. It MUST run before `apply_state_snapshot` (which sits in
    // `GuestSnapshotSet`) — otherwise the snapshot's CRDT merge re-
    // derives `Health.current` from the host's stale view (no guest
    // damage recorded yet), erasing the just-dealt damage for a frame
    // until the next round-trip. The `.before(GuestSnapshotSet)` is a
    // soft constraint: vacuous on the host (no member systems), enforced
    // on the guest.
    app.add_systems(
        Update,
        crdt_sync::sync_health_to_crdt
            .after(PostCombatSet)
            .before(crate::game::units::GuestSnapshotSet)
            .run_if(both_peers.clone()),
    );
    app.add_systems(
        Update,
        crdt_sync::receive_wall_placement.run_if(both_peers.clone()),
    );

    // ── Wizard Spell-Stat Tally (both MP peers) ──────────────────
    // Sums the per-frame spell damage/heal tally markers into the local
    // wizard's `LocalWizardStats` and clears them. Runs on both peers in
    // multiplayer (each accumulates its own wizard's output). Deliberately
    // NOT registered for single-player: there the tally markers are emitted
    // but never consumed (they stay inert on the unit and clear on despawn),
    // so single-player pays no per-frame cost.
    app.add_systems(
        Update,
        score_stats::accumulate_wizard_spell_stats.run_if(both_peers.clone()),
    );

    // ── Host: MP King Death Check (VERSUS only) ──────────────────
    // Replaces SP's check_win_lose_conditions during a versus match. The
    // co-op host uses SP's `check_win_lose_conditions` natively (it's in
    // InGame), so this stays `in_mp_running`-gated and must NOT be widened.
    let mp_host = in_mp_running.and_then(is_multiplayer_host);

    app.add_systems(
        Update,
        host_systems::check_mp_king_death
            .after(PostCombatSet)
            // Run after the tally accumulator so the killing blow's spell
            // damage/healing is folded into LocalWizardStats before the
            // host snapshots it into the game-over summary.
            .after(score_stats::accumulate_wizard_spell_stats)
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
            .run_if(host_net.clone()),
    );
}
