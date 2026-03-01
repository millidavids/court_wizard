use bevy::prelude::*;

use super::constants::*;

/// Resource that controls when eyes transfer between hags.
#[derive(Resource)]
pub struct EyeTransferTimer {
    pub time_remaining: f32,
}

/// Resource tracking permanently dead hags.
#[derive(Resource)]
pub struct HagDeathTracker {
    pub permanent_deaths: u32,
}

impl HagDeathTracker {
    pub fn new() -> Self {
        Self {
            permanent_deaths: 0,
        }
    }
}

/// Pre-loaded meshes and materials for the hag bosses.
#[derive(Resource)]
pub struct HagAssets {
    pub mesh: Handle<Mesh>,
    pub justina_material: Handle<StandardMaterial>,
    pub martina_material: Handle<StandardMaterial>,
    pub josephina_material: Handle<StandardMaterial>,
    pub eye_mesh: Handle<Mesh>,
    pub invulnerability_eye_material: Handle<StandardMaterial>,
    pub ability_eye_material: Handle<StandardMaterial>,
    pub mind_control_aura_mesh: Handle<Mesh>,
    pub mind_control_aura_material: Handle<StandardMaterial>,
}

/// System to pre-load hag assets at startup.
pub(super) fn preload_hag_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = HagAssets {
        mesh: meshes.add(Ellipse::new(HAG_ELLIPSE_WIDTH, HAG_ELLIPSE_DEPTH)),
        justina_material: materials.add(StandardMaterial {
            base_color: JUSTINA_COLOR,
            unlit: true,
            ..default()
        }),
        martina_material: materials.add(StandardMaterial {
            base_color: MARTINA_COLOR,
            unlit: true,
            ..default()
        }),
        josephina_material: materials.add(StandardMaterial {
            base_color: JOSEPHINA_COLOR,
            unlit: true,
            ..default()
        }),
        eye_mesh: meshes.add(Circle::new(EYE_VISUAL_RADIUS)),
        invulnerability_eye_material: materials.add(StandardMaterial {
            base_color: INVULNERABILITY_EYE_COLOR,
            unlit: true,
            ..default()
        }),
        ability_eye_material: materials.add(StandardMaterial {
            base_color: ABILITY_EYE_COLOR,
            unlit: true,
            ..default()
        }),
        mind_control_aura_mesh: meshes.add(Circle::new(MIND_CONTROL_AURA_RADIUS)),
        mind_control_aura_material: materials.add(StandardMaterial {
            base_color: MIND_CONTROL_AURA_COLOR,
            unlit: true,
            alpha_mode: bevy::prelude::AlphaMode::Blend,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
