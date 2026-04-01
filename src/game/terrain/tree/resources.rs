use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for trees.
#[derive(Resource)]
pub struct TreeAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load tree assets at startup.
pub(super) fn preload_tree_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = TreeAssets {
        mesh: meshes.add(Rectangle::new(TREE_VISUAL_WIDTH, TREE_VISUAL_HEIGHT)),
        material: materials.add(StandardMaterial {
            base_color: TREE_COLOR,
            unlit: true,
            ..default()
        }),
    };
    commands.insert_resource(assets);
}
