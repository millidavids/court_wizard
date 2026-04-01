use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for bushes.
#[derive(Resource)]
pub struct BushAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load bush assets at startup.
pub(super) fn preload_bush_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = BushAssets {
        mesh: meshes.add(Circle::new(BUSH_VISUAL_RADIUS)),
        material: materials.add(StandardMaterial {
            base_color: BUSH_COLOR,
            unlit: true,
            ..default()
        }),
    };
    commands.insert_resource(assets);
}
