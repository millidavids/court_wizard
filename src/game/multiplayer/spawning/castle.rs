//! Multiplayer castle wall spawn helper.

use bevy::prelude::*;

use crate::game::battlefield::components::BattlefieldAssets;

/// Spawns a castle wall plane at the given position and rotation.
///
/// Tagged `OnGameplayScreen` to match the castle that `setup_battlefield`
/// spawns (Castle 1) — both castles share one cleanup marker so any future
/// query that looks them up by marker finds both. `origin_transform` is the
/// same per-client visual mirror passed to `setup_battlefield`.
#[allow(clippy::too_many_arguments)]
pub(in crate::game::multiplayer) fn spawn_castle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    battlefield_assets: &BattlefieldAssets,
    position: Vec3,
    rotation_degrees: f32,
    origin_transform: Transform,
) {
    crate::game::battlefield::systems::spawn_castle_wall(
        commands,
        meshes,
        materials,
        battlefield_assets,
        position,
        rotation_degrees,
        crate::game::components::OnGameplayScreen,
        origin_transform,
    );
}
