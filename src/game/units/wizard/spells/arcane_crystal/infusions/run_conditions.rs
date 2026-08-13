//! Run conditions gating the per-infusion systems.
//!
//! Mirrors the `spell_is_primed` factory in `spells/run_conditions.rs`.

use bevy::prelude::*;

use super::super::components::ArcaneCrystal;
use super::kinds::CrystalInfusion;

/// True when **any** live crystal currently holds `infusion`.
///
/// This is a cheap gate, never a selector. Crystal Network allows up to
/// `CRYSTAL_NETWORK_MAX_CRYSTALS` crystals at once, so several differently-infused
/// crystals can satisfy several different gates in the same frame. Every infusion
/// system must still filter per-crystal — see [`is_infused`].
pub(crate) fn crystal_infused_with(
    infusion: CrystalInfusion,
) -> impl Fn(
    Query<&ArcaneCrystal, Without<crate::game::multiplayer::components::GhostSpellEffect>>,
) -> bool
+ Clone {
    // Matches what the systems behind this gate actually iterate. Without the
    // ghost exclusion the guest's mirrored copy of the host's crystal — whose
    // infusion is synced every frame — would open the gate for systems whose own
    // queries then match nothing.
    move |crystals: Query<
        &ArcaneCrystal,
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >| {
        crystals
            .iter()
            .any(|crystal| !crystal.permanent && crystal.infusion == Some(infusion))
    }
}

/// Per-crystal check an infusion system applies inside its loop, so a crystal
/// holding a different infusion is skipped.
///
/// Also rejects Auto-Crystal turrets, which never take an infusion at all.
pub(crate) fn is_infused(crystal: &ArcaneCrystal, infusion: CrystalInfusion) -> bool {
    !crystal.permanent && crystal.infusion == Some(infusion)
}
