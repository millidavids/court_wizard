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
    pub charge_rect_mesh: Handle<Mesh>,
    pub charge_line_material: Handle<StandardMaterial>,
    pub charge_fill_material: Handle<StandardMaterial>,
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
        charge_rect_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        charge_line_material: materials.add(StandardMaterial {
            base_color: OGRE_CHARGE_LINE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        charge_fill_material: materials.add(StandardMaterial {
            base_color: OGRE_CHARGE_FILL_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
