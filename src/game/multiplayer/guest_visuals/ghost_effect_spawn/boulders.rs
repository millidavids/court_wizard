use bevy::prelude::*;

use crate::networking::snapshot::SpellEffectSnapshot;

use crate::game::multiplayer::components::OnMultiplayerGameScreen;

pub(crate) fn spawn_boulder_projectile(
    commands: &mut Commands,
    _effect: &SpellEffectSnapshot,
    pos: Vec3,
    boulder_assets: &crate::game::terrain::boulder::resources::BoulderAssets,
) -> Option<Entity> {
    // Spawn a ghost mid-air boulder at the host's transform; the
    // host's per-frame `NetworkedSpellEffect` snapshot keeps it on
    // the same arc. `extra[0]` carries the sprite index so the
    // guest picks the matching boulder material.
    let sprite_index = _effect.extra[0] as usize;
    let idx = sprite_index.min(boulder_assets.materials.len().saturating_sub(1));
    Some(
        commands
            .spawn((
                Mesh3d(boulder_assets.mesh.clone()),
                MeshMaterial3d(boulder_assets.materials[idx].clone()),
                Transform::from_translation(pos),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_boulder_obstacle(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    boulder_assets: &crate::game::terrain::boulder::resources::BoulderAssets,
) -> Option<Entity> {
    // Spawn a ghost grounded boulder at the host's land position.
    // `extra[0]` = sprite_index, `extra[1]` = radius, `extra[2]` =
    // height. Billboard so the sprite faces the camera identically
    // to the SP path. No `Boulder` / `ObstacleHealth` component on
    // the ghost — those are host-authoritative and would re-trigger
    // gameplay systems if present (they're gated `is_gameplay_running`
    // = host-only, so they wouldn't actually fire on the guest, but
    // leaving them off is cleaner).
    let sprite_index = effect.extra[0] as usize;
    let idx = sprite_index.min(boulder_assets.materials.len().saturating_sub(1));
    Some(
        commands
            .spawn((
                Mesh3d(boulder_assets.mesh.clone()),
                MeshMaterial3d(boulder_assets.materials[idx].clone()),
                Transform::from_translation(pos),
                crate::game::components::Billboard,
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}
