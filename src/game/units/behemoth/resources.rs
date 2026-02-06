use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for behemoth units.
#[derive(Resource)]
pub struct BehemothAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load behemoth assets at startup.
pub(super) fn preload_behemoth_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = BehemothAssets {
        mesh: meshes.add(Ellipse::new(BEHEMOTH_ELLIPSE_WIDTH, BEHEMOTH_ELLIPSE_DEPTH)),
        material: materials.add(StandardMaterial {
            base_color: BEHEMOTH_COLOR,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
