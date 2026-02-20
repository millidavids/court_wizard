use bevy::prelude::*;

use crate::game::constants::KING_CORPSE_COLOR;

use super::constants::*;

/// Pre-loaded meshes and materials for the king unit.
#[derive(Resource)]
pub struct KingAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub corpse_material: Handle<StandardMaterial>,
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
        corpse_material: materials.add(StandardMaterial {
            base_color: KING_CORPSE_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    };

    commands.insert_resource(assets);
}
