//! Per-unit snapshot types and builder helpers.

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::game::units::components::{Health, Team};

use crate::networking::entity_map::NetworkEntityId;

/// Per-unit network snapshot.
///
/// **HP via CRDT, not via `health_pct`:** the CRDT damage[2]/healing[2]
/// slots are the source of truth for HP — they support both peers
/// applying damage AND healing to the same unit and converging via
/// element-wise max merge. `Health.current` is re-derived from CRDT on
/// each peer. The previous `health_pct` field is dropped because it was
/// redundant with the CRDT-derived value.
///
/// **Wire-format note:** bincode 1.x encodes fields positionally with no
/// version tags, so adding/removing/reordering any field is a breaking
/// change — mixed-version peers silently misread later fields as garbage
/// rather than failing fast. MP currently requires same-version peers.
///
/// Velocity is XZ-only (`vx`, `vz`).
#[derive(Serialize, Deserialize)]
pub struct UnitSnapshot {
    /// Network entity ID assigned by the host.
    pub id: u32,
    /// World position X.
    pub x: f32,
    /// World position Y (height above battlefield).
    pub y: f32,
    /// World position Z.
    pub z: f32,
    /// Velocity X (host-authoritative, used by guest animation systems).
    pub vx: f32,
    /// Velocity Z (host-authoritative, used by guest animation systems).
    pub vz: f32,
    /// Team encoded as u8: 0=Defenders, 1=Attackers, 2=Undead.
    pub team: u8,
    /// Bitfield flags (see `UnitFlags`). Widened u8→u16 when COMBAT_ANIMATION
    /// was added, then u16→u32 when the buff-visual bits (protocol v13) used
    /// up the last free bit.
    pub flags: u32,
    /// Max HP — needed once but ships every frame (CRDT seed; could be
    /// thinned later via a separate one-shot spawn message).
    pub max_hp: f32,
    /// CRDT damage counters per peer (monotonically increasing). Both
    /// peers apply damage to their local Health → `sync_health_to_crdt`
    /// records into their own slot → snapshots cross-fertilise → merge
    /// = element-wise max so both sides converge.
    pub damage: [f32; 2],
    /// CRDT healing counters per peer (monotonically increasing).
    pub healing: [f32; 2],
}

/// Bitfield constants for `UnitSnapshot::flags`.
pub struct UnitFlags;

impl UnitFlags {
    pub const CORPSE: u32 = 1 << 0;
    pub const KING: u32 = 1 << 1;
    pub const ARCHER: u32 = 1 << 2;
    pub const KINGS_GUARD: u32 = 1 << 3;
    pub const FIRE_EFFECT: u32 = 1 << 4;
    pub const FROST_EFFECT: u32 = 1 << 5;
    pub const ELECTRIC_EFFECT: u32 = 1 << 6;
    pub const SPELL_SHIELD: u32 = 1 << 7;
    /// Host's unit currently has a `CombatAnimation` component — set so
    /// the guest can spawn the matching swing/shoot animation on its
    /// ghost. Without this the ghost stays on the idle frame even though
    /// the host's unit is actively swinging in melee.
    pub const COMBAT_ANIMATION: u32 = 1 << 8;
    /// Host's unit currently carries Mark of Death — set so the guest renders
    /// the floating mark indicator on its ghost unit. Without this, marks the
    /// HOST casts never reach the guest (the guest only knows about marks it
    /// cast itself and applied locally).
    pub const MARK_EFFECT: u32 = 1 << 9;
    /// Host's unit is poisoned (Plague Wind etc.) — set so the guest renders
    /// the green poison tint on its ghost unit.
    pub const POISON_EFFECT: u32 = 1 << 10;
    /// Host's unit is currently polymorphed (a sheep) — set so the guest swaps
    /// its ghost to the sheep sprite. Without this a host-cast polymorph never
    /// renders on the guest (the unit snapshot otherwise carries no "is a sheep"
    /// state and the guest only swaps materials at spawn / corpse transitions).
    pub const POLYMORPH: u32 = 1 << 11;
    /// Host's unit IS the Swordcerer battlefield avatar — set so the guest spawns
    /// the ghost with the avatar sprite/hitbox and renders its health bar, rather
    /// than treating it as a generic infantry unit.
    pub const SWORDCERER_AVATAR: u32 = 1 << 12;
    /// Host's unit is currently smelly (Excremage poop debuff) — set so the guest
    /// renders the brown stink tint on its ghost unit.
    pub const SMELLY: u32 = 1 << 13;
    /// Host's unit is engaged in melee — set so the guest can run the battle
    /// ambience (melee-sound) loop scaled by the on-field combat it can hear.
    pub const IN_MELEE: u32 = 1 << 14;
    /// Host's unit is enraged (`BerserkerRageModifier`) — set so the guest
    /// renders the red rage tint on its ghost unit.
    pub const BERSERKER_RAGE: u32 = 1 << 15;
    /// Host's unit is under Battle Hymn — set so the guest renders the
    /// rising song-motes on its ghost unit.
    pub const BATTLE_HYMN: u32 = 1 << 16;
    /// Host's unit has temporary hit points (Guardian Circle etc.) — set so
    /// the guest renders the feet-ring shield indicator on its ghost unit.
    pub const TEMP_HP: u32 = 1 << 17;
    /// Host's unit is hasted — set so the guest renders speed lines on its
    /// ghost unit while it moves.
    pub const HASTE: u32 = 1 << 18;
    /// Host's unit was recently healed (`RecentlyHealedVfx`) — set so the
    /// guest renders the green regen motes on its ghost unit.
    pub const HEALING: u32 = 1 << 19;
}

/// Encodes a `Team` component into a u8.
pub fn team_to_u8(team: &Team) -> u8 {
    match team {
        Team::Defenders => 0,
        Team::Attackers => 1,
        Team::Undead => 2,
    }
}

/// Decodes a u8 back into a `Team`.
pub fn u8_to_team(val: u8) -> Team {
    match val {
        0 => Team::Defenders,
        1 => Team::Attackers,
        _ => Team::Undead,
    }
}

/// Builds a `UnitSnapshot` from an entity's components.
///
/// HP rides in the CRDT damage/healing arrays (NOT a redundant `health_pct`)
/// so both peers can apply damage AND healing and converge via element-wise
/// max. `health_pct` is gone — guest derives `Health.current` from CRDT.
///
/// `flags` is a pre-packed `UnitFlags` bitfield — the caller ORs the bits
/// together where the `Has<T>` query results are named, instead of threading
/// 20+ positional bools through this signature.
pub fn build_unit_snapshot(
    net_id: &NetworkEntityId,
    transform: &Transform,
    velocity: &crate::game::components::Velocity,
    team: &Team,
    health: &Health,
    crdt_health: Option<&crate::networking::crdt::CrdtHealth>,
    flags: u32,
) -> UnitSnapshot {
    let (max_hp, damage, healing) = if let Some(crdt) = crdt_health {
        (crdt.max_hp, crdt.damage, crdt.healing)
    } else {
        // First snapshot before `attach_crdt_health` has run — derive
        // initial CRDT state from `Health` (all loss attributed to peer 0).
        let total_damage = (health.max - health.current).max(0.0);
        (health.max, [total_damage, 0.0], [0.0, 0.0])
    };

    UnitSnapshot {
        id: net_id.0,
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        vx: velocity.x,
        vz: velocity.z,
        team: team_to_u8(team),
        flags,
        max_hp,
        damage,
        healing,
    }
}

/// Monotonically increasing tick counter for snapshot ordering.
#[derive(Resource, Default)]
pub struct SnapshotTick(pub u32);
