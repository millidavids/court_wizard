use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::units::archer::ArcherAssets;
use crate::networking::snapshot::ArrowSnapshot;

use super::super::super::components::{GhostArrow, OnMultiplayerGameScreen};

/// Despawns all existing ghost arrows and respawns them at the positions
/// reported in the snapshot. Called once per snapshot frame.
pub(super) fn sync_ghost_arrows(
    commands: &mut Commands,
    ghost_arrows: &Query<Entity, With<GhostArrow>>,
    archer_assets: &Res<ArcherAssets>,
    arrows: &[ArrowSnapshot],
) {
    for entity in ghost_arrows {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }

    for arrow in arrows {
        commands.spawn((
            Mesh3d(archer_assets.arrow_mesh.clone()),
            MeshMaterial3d(archer_assets.arrow_material.clone()),
            Transform::from_translation(Vec3::new(arrow.x, arrow.y, arrow.z)),
            Billboard,
            GhostArrow,
            OnMultiplayerGameScreen,
        ));
    }
}
