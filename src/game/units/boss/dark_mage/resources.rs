use bevy::prelude::*;

use super::constants::*;

/// Pre-loaded meshes and materials for the Dark Mage boss.
#[derive(Resource)]
pub struct DarkMageAssets {
    /// Body mesh (ellipse).
    pub mesh: Handle<Mesh>,
    /// Body materials per enrage phase.
    pub material_phase0: Handle<StandardMaterial>,
    pub material_phase1: Handle<StandardMaterial>,
    pub material_phase2: Handle<StandardMaterial>,
    pub material_phase3: Handle<StandardMaterial>,
    /// Circle indicator mesh (unit circle, scaled per-spell).
    pub circle_mesh: Handle<Mesh>,
    /// Rectangle mesh for lightning corridor indicator (1x1, scaled).
    pub rect_mesh: Handle<Mesh>,
    /// Meteor explosion circle mesh.
    pub explosion_mesh: Handle<Mesh>,
    /// Plague cloud zone material.
    pub plague_zone_material: Handle<StandardMaterial>,
    /// Lightning strike material.
    pub lightning_strike_material: Handle<StandardMaterial>,
    /// Meteor explosion material.
    pub meteor_explosion_material: Handle<StandardMaterial>,
}

/// System to pre-load Dark Mage assets at startup.
pub(super) fn preload_dark_mage_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = DarkMageAssets {
        mesh: meshes.add(Ellipse::new(DARK_MAGE_ELLIPSE_WIDTH, DARK_MAGE_ELLIPSE_DEPTH)),
        material_phase0: materials.add(StandardMaterial {
            base_color: DARK_MAGE_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase1: materials.add(StandardMaterial {
            base_color: DARK_MAGE_ENRAGE_1_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase2: materials.add(StandardMaterial {
            base_color: DARK_MAGE_ENRAGE_2_COLOR,
            unlit: true,
            ..default()
        }),
        material_phase3: materials.add(StandardMaterial {
            base_color: DARK_MAGE_ENRAGE_3_COLOR,
            unlit: true,
            ..default()
        }),
        circle_mesh: meshes.add(Circle::new(1.0)),
        rect_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        explosion_mesh: meshes.add(Circle::new(1.0)),
        plague_zone_material: materials.add(StandardMaterial {
            base_color: PLAGUE_ZONE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        lightning_strike_material: materials.add(StandardMaterial {
            base_color: LIGHTNING_FILL_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        meteor_explosion_material: materials.add(StandardMaterial {
            base_color: METEOR_FILL_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
