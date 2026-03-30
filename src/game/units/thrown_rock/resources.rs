use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for thrown rocks.
#[derive(Resource)]
pub struct ThrownRockAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// System to pre-load rock assets at startup.
pub(super) fn preload_thrown_rock_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = ThrownRockAssets {
        mesh: meshes.add(Circle::new(ROCK_VISUAL_RADIUS)),
        material: materials.add(StandardMaterial {
            base_color: ROCK_BASE_COLOR,
            unlit: true,
            ..default()
        }),
    };
    commands.insert_resource(assets);
}
