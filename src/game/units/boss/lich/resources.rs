use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for the Lich boss.
#[derive(Resource)]
pub struct LichAssets {
    pub mesh: Handle<Mesh>,
    pub material_summoning: Handle<StandardMaterial>,
    pub material_combat: Handle<StandardMaterial>,
}

/// System to pre-load Lich assets at startup.
pub(super) fn preload_lich_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = LichAssets {
        mesh: meshes.add(Ellipse::new(LICH_ELLIPSE_WIDTH, LICH_ELLIPSE_DEPTH)),
        material_summoning: materials.add(StandardMaterial {
            base_color: LICH_COLOR,
            unlit: true,
            ..default()
        }),
        material_combat: materials.add(StandardMaterial {
            base_color: LICH_COMBAT_COLOR,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
