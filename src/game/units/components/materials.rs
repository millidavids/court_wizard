use bevy::prelude::*;

/// Stores the original shared material handle before persistent effect tinting.
///
/// Inserted when a persistent damage effect (FireDoT, FrostEffectMarker, Shocked)
/// is first applied to a unit. The unit's MeshMaterial3d is replaced with a cloned
/// per-entity material that can be safely tinted without affecting other units.
/// When all effects expire, the original material is restored and this component is removed.
#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

/// Visual-only markers for status effects active on the remote peer.
///
/// These are inserted/removed based on network snapshot flags so that
/// `update_persistent_effect_visuals` can tint units without creating
/// damage-ticking DoT components (which would double-count damage via CRDT).
#[derive(Component)]
pub struct RemoteFireEffect;

#[derive(Component)]
pub struct RemoteFrostEffect;

#[derive(Component)]
pub struct RemoteElectricEffect;

/// Visual-only poison marker, mirrored from the host's `PoisonedModifier` via
/// `UnitFlags::POISON_EFFECT`. Drives the green tint in
/// `update_persistent_effect_visuals` without inserting a damage-ticking
/// `PoisonedModifier` (which would double-count poison DoT on the guest).
#[derive(Component)]
pub struct RemotePoisonEffect;

/// Visual-only polymorph marker, mirrored from the host's `PolymorphedModifier`
/// via `UnitFlags::POLYMORPH`. Tracks that a guest ghost is currently rendered as
/// a sheep so the snapshot loop swaps the mesh/material to the sheep sprite on the
/// off→on edge and restores the original sprite on the on→off edge. No gameplay
/// component is created on the guest — the sheep state is host-authoritative.
#[derive(Component)]
pub struct RemotePolymorphEffect;
