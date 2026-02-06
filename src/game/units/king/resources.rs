use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for the king unit.
#[derive(Resource)]
pub struct KingAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load king assets at startup.
pub(super) fn preload_king_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = KingAssets {
        mesh: meshes.add(Circle::new(KING_RADIUS)),
        material: materials.add(StandardMaterial {
            base_color: KING_COLOR,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
