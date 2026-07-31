//! Status-effect flag plumbing shared by the ghost spawn/update paths.
//!
//! Two views of the same set of effects are diffed each snapshot to detect
//! on/off edges: [`RemoteEffectFlags`] is what the host's snapshot says
//! *should* be true, [`GhostMarkerState`] is what the ghost's ECS state
//! currently *is*. Keeping them as distinct types is deliberate — conflating
//! wire truth with local truth is the source of this codebase's most common
//! multiplayer bugs.

use bevy::prelude::*;

use crate::networking::snapshot::UnitFlags;

/// Status-effect flags for one unit, decoded from `UnitSnapshot.flags`.
///
/// Bundled into a struct so the per-ghost update/spawn helpers don't take a
/// parameter per flag (they were pushing 40+ raw parameters).
pub(super) struct RemoteEffectFlags {
    pub fire: bool,
    pub frost: bool,
    pub electric: bool,
    pub spell_shield: bool,
    pub combat: bool,
    pub poison: bool,
    pub mark: bool,
    pub polymorph: bool,
    pub smelly: bool,
    pub in_melee: bool,
    pub rage: bool,
    pub battle_hymn: bool,
    pub temp_hp: bool,
    pub haste: bool,
    pub healing: bool,
}

impl RemoteEffectFlags {
    /// Decodes the status-effect bits of a `UnitSnapshot.flags` value.
    pub(super) fn from_flags(flags: u32) -> Self {
        Self {
            fire: flags & UnitFlags::FIRE_EFFECT != 0,
            frost: flags & UnitFlags::FROST_EFFECT != 0,
            electric: flags & UnitFlags::ELECTRIC_EFFECT != 0,
            spell_shield: flags & UnitFlags::SPELL_SHIELD != 0,
            combat: flags & UnitFlags::COMBAT_ANIMATION != 0,
            poison: flags & UnitFlags::POISON_EFFECT != 0,
            mark: flags & UnitFlags::MARK_EFFECT != 0,
            polymorph: flags & UnitFlags::POLYMORPH != 0,
            smelly: flags & UnitFlags::SMELLY != 0,
            in_melee: flags & UnitFlags::IN_MELEE != 0,
            rage: flags & UnitFlags::BERSERKER_RAGE != 0,
            battle_hymn: flags & UnitFlags::BATTLE_HYMN != 0,
            temp_hp: flags & UnitFlags::TEMP_HP != 0,
            haste: flags & UnitFlags::HASTE != 0,
            healing: flags & UnitFlags::HEALING != 0,
        }
    }
}

/// Which effect markers a ghost entity currently carries (read from the ghost
/// query), used to detect on/off edges against the snapshot's
/// `RemoteEffectFlags`.
pub(super) struct GhostMarkerState {
    pub fire: bool,
    pub frost: bool,
    pub electric: bool,
    pub spell_shield: bool,
    pub corpse: bool,
    pub combat: bool,
    pub poison: bool,
    pub mark: bool,
    pub polymorph: bool,
    pub rage: bool,
    pub battle_hymn: bool,
    pub temp_hp: bool,
    pub haste: bool,
    pub healing: bool,
}

/// Inserts `marker` on the snapshot flag's off→on edge and removes `T` on the
/// on→off edge; no-ops while the flag and marker already agree.
pub(super) fn sync_remote_marker<T: Component>(
    commands: &mut Commands,
    entity: Entity,
    has_marker: bool,
    flag_set: bool,
    marker: T,
) {
    if flag_set && !has_marker {
        commands.entity(entity).insert(marker);
    } else if !flag_set && has_marker {
        commands.entity(entity).remove::<T>();
    }
}
