use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for the ogre boss.
/// Includes materials for each enrage phase.
#[derive(Resource)]
pub struct OgreAssets {
    pub mesh: Handle<Mesh>,
    pub material_phase0: Handle<StandardMaterial>,
    pub material_phase1: Handle<StandardMaterial>,
    pub material_phase2: Handle<StandardMaterial>,
    pub material_phase3: Handle<StandardMaterial>,
}

/// System to pre-load ogre assets at startup.
pub(super) fn preload_ogre_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = OgreAssets {
        mesh: meshes.add(Ellipse::new(OGRE_ELLIPSE_WIDTH, OGRE_ELLIPSE_DEPTH)),
        material_phase0: materials.add(StandardMaterial {
            base_color: OGRE_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase1: materials.add(StandardMaterial {
            base_color: OGRE_ENRAGE_1_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase2: materials.add(StandardMaterial {
            base_color: OGRE_ENRAGE_2_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase3: materials.add(StandardMaterial {
            base_color: OGRE_ENRAGE_3_COLOR,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
