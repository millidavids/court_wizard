use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for boulders.
#[derive(Resource)]
pub struct BoulderAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load boulder assets at startup.
pub(super) fn preload_boulder_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = BoulderAssets {
        mesh: meshes.add(Circle::new(ROCK_VISUAL_RADIUS)),
        material: materials.add(StandardMaterial {
            base_color: ROCK_BASE_COLOR,
            unlit: true,
            ..default()
        }),
    };
    commands.insert_resource(assets);
}
