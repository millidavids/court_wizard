use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;

use crate::game::constants::SPELL_ORIGIN;
use crate::game::units::components::Team;
use crate::networking::resources::PeerRole;
use crate::networking::session::MultiplayerSession;

/// World-space position the **local** player's spells originate from. Single-
/// player and the multiplayer host use `SPELL_ORIGIN`; the multiplayer guest
/// uses `SPELL_2_ORIGIN`. Set at app startup to the SP default and overridden
/// by `init_mp_game` based on the peer's role.
#[derive(Resource, Debug, Clone, Copy)]
pub struct LocalSpellOrigin(pub Vec3);

impl Default for LocalSpellOrigin {
    fn default() -> Self {
        Self(SPELL_ORIGIN)
    }
}

/// The team the **local** player commands: `Defenders` in single-player, as the
/// multiplayer host, and as a **co-op** guest (co-op partners are both
/// Defenders); `Attackers` only as the **versus** guest. Enemy-targeting spells
/// should filter with `unit_team.is_enemy(&local_player_team(session.as_deref()))`
/// instead of hardcoding `Team::Attackers` (which only resolves correctly for the
/// host, making a guest target its own army).
pub fn local_player_team(session: Option<&MultiplayerSession>) -> Team {
    match session {
        // Versus guest commands the Attackers army; everyone else (SP, host,
        // co-op guest) commands the Defenders.
        Some(s) if s.role == PeerRole::Guest && !s.is_coop() => Team::Attackers,
        _ => Team::Defenders,
    }
}

// ----- Lock-free snapshot of the local spell origin for non-system callers -----
//
// Some helper code paths (audio attenuation, const-derived gunslinger spawn
// positions) cannot easily take `Res<LocalSpellOrigin>` as a system param
// because they are either plain functions or const-context expressions. They
// read this snapshot instead. `LocalSpellOriginSyncPlugin` keeps it updated
// from the `LocalSpellOrigin` resource so the multiplayer guest sees their
// own wizard's value.

// Initialised directly to `SPELL_ORIGIN`'s bit pattern so the snapshot is
// already correct for SP / MP host before any system runs. The previous
// `Once`-based lazy init had a race: if `update_local_spell_origin_snapshot`
// wrote SPELL_2_ORIGIN first (guest), a later reader's `call_once` would
// stomp it back to SPELL_ORIGIN — permanently, for the rest of the process.
static LOCAL_ORIGIN_X: AtomicU32 = AtomicU32::new(SPELL_ORIGIN.x.to_bits());
static LOCAL_ORIGIN_Y: AtomicU32 = AtomicU32::new(SPELL_ORIGIN.y.to_bits());
static LOCAL_ORIGIN_Z: AtomicU32 = AtomicU32::new(SPELL_ORIGIN.z.to_bits());

pub(crate) fn set_local_spell_origin_snapshot(pos: Vec3) {
    LOCAL_ORIGIN_X.store(pos.x.to_bits(), Ordering::Relaxed);
    LOCAL_ORIGIN_Y.store(pos.y.to_bits(), Ordering::Relaxed);
    LOCAL_ORIGIN_Z.store(pos.z.to_bits(), Ordering::Relaxed);
}

pub(crate) fn local_spell_origin_snapshot() -> Vec3 {
    Vec3::new(
        f32::from_bits(LOCAL_ORIGIN_X.load(Ordering::Relaxed)),
        f32::from_bits(LOCAL_ORIGIN_Y.load(Ordering::Relaxed)),
        f32::from_bits(LOCAL_ORIGIN_Z.load(Ordering::Relaxed)),
    )
}

/// Bevy system: pushes the current `LocalSpellOrigin` resource into the
/// lock-free snapshot whenever it changes.
pub(crate) fn update_local_spell_origin_snapshot(local_origin: Res<LocalSpellOrigin>) {
    set_local_spell_origin_snapshot(local_origin.0);
}
