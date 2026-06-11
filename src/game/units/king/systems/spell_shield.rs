use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::resources::InitialDefenderCount;
use crate::game::units::components::{Corpse, Team};
use crate::networking::session::MultiplayerSession;

/// Attaches the `SpellShield` marker component to every King that appears in
/// a multiplayer match. Runs once per king on the host (via `Added<King>`).
///
/// **Host only.** The guest must NOT independently attach SpellShield —
/// shield state is host-authoritative and propagates to the guest via the
/// `apply_state_snapshot` spell-shield transition handler. If the guest ran
/// this too, it would insert SpellShield on every ghost king the frame after
/// the ghost gets its `King` marker, racing with (and briefly overriding)
/// the snapshot's intended state.
///
/// No separate visual is spawned here — the king's aura (added by the spawn
/// path; see `spawn_king_aura_visual`) is the constant visual.
pub fn attach_king_spell_shield(
    mut commands: Commands,
    session: Option<Res<MultiplayerSession>>,
    new_kings: Query<Entity, Added<King>>,
) {
    use crate::networking::resources::PeerRole;
    // VERSUS only. The spell shield is a duel mechanic — it must NOT appear in
    // endless/roguelite, including CO-OP endless/roguelite (where the host holds
    // a co-op session but plays the single-player battlefield). Single-player has
    // no session, so it's already excluded.
    if !session.is_some_and(|s| s.role == PeerRole::Host && !s.is_coop()) {
        return;
    }

    for entity in &new_kings {
        commands.entity(entity).insert(SpellShield);
    }
}

/// Removes a King's spell shield when fewer than 10% of their own team's
/// non-King units remain alive. Iterates per-king so both kings in MP get
/// independent shield-degradation tracking. The aura visual is unrelated
/// and stays on as long as the king lives — this only touches the marker.
pub fn update_king_spell_shield(
    mut commands: Commands,
    kings: Query<(Entity, &Team), (With<King>, With<SpellShield>, Without<Corpse>)>,
    units: Query<&Team, (Without<Corpse>, Without<King>)>,
    initial_count: Option<Res<InitialDefenderCount>>,
    session: Option<Res<MultiplayerSession>>,
) {
    // Initial-count source differs by mode:
    // - SP: `InitialDefenderCount` is set by the SP loader to the actual
    //   defender army size, which scales with progression. Only the
    //   Defender team has a king in SP, so a single threshold suffices.
    // - MP: both teams start with a known, symmetric army size; we use the
    //   compile-time MP constants instead of plumbing per-team resources.
    let initial = if session.is_some() {
        use crate::game::constants::{KINGS_GUARD_COUNT, MP_ARCHER_COUNT, MP_INFANTRY_COUNT};
        (MP_INFANTRY_COUNT + MP_ARCHER_COUNT + KINGS_GUARD_COUNT) as f32
    } else if let Some(c) = initial_count {
        c.0 as f32
    } else {
        return;
    };
    if initial <= 0.0 {
        return;
    }

    for (king_entity, team) in &kings {
        let living = units.iter().filter(|t| *t == team).count() as f32;
        if living / initial <= SPELL_SHIELD_THRESHOLD {
            commands.entity(king_entity).remove::<SpellShield>();
        }
    }
}

/// Multiplayer anti-stall: force-removes the King's spell shield once the match
/// has run `MP_SPELL_SHIELD_MAX_DURATION_SECS` seconds, regardless of how many
/// units are still alive. Without this, a player could maze / play keep-away to
/// keep enough units alive that the kill-threshold drop (`update_king_spell_shield`)
/// never fires, stalling the match forever.
///
/// **Host only** (registered under `is_gameplay_running`): shield state is
/// host-authoritative and propagates to the guest via the snapshot `SPELL_SHIELD`
/// bit. Reuses the host-side `KillStats.elapsed_time` match clock (reset at match
/// start), so no extra timer is needed. Removes the shield from every still-shielded
/// king, so both MP kings lose their shields together at the timeout.
pub fn expire_king_spell_shield_on_timeout(
    mut commands: Commands,
    kill_stats: Res<crate::game::resources::KillStats>,
    kings: Query<Entity, (With<King>, With<SpellShield>, Without<Corpse>)>,
) {
    if kill_stats.elapsed_time >= MP_SPELL_SHIELD_MAX_DURATION_SECS {
        for king_entity in &kings {
            commands.entity(king_entity).remove::<SpellShield>();
        }
    }
}
