use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for brute units.
#[derive(Resource)]
pub struct BruteAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load brute assets at startup.
pub(super) fn preload_brute_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = BruteAssets {
        mesh: meshes.add(Ellipse::new(BRUTE_ELLIPSE_WIDTH, BRUTE_ELLIPSE_DEPTH)),
        material: materials.add(StandardMaterial {
            base_color: BRUTE_COLOR,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
