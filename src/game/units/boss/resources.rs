use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for the boss unit.
/// Includes materials for each enrage phase.
#[derive(Resource)]
pub struct BossAssets {
    pub mesh: Handle<Mesh>,
    pub material_phase0: Handle<StandardMaterial>,
    pub material_phase1: Handle<StandardMaterial>,
    pub material_phase2: Handle<StandardMaterial>,
    pub material_phase3: Handle<StandardMaterial>,
}

/// System to pre-load boss assets at startup.
pub(super) fn preload_boss_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = BossAssets {
        mesh: meshes.add(Ellipse::new(BOSS_ELLIPSE_WIDTH, BOSS_ELLIPSE_DEPTH)),
        material_phase0: materials.add(StandardMaterial {
            base_color: BOSS_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase1: materials.add(StandardMaterial {
            base_color: BOSS_ENRAGE_1_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase2: materials.add(StandardMaterial {
            base_color: BOSS_ENRAGE_2_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase3: materials.add(StandardMaterial {
            base_color: BOSS_ENRAGE_3_COLOR,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
