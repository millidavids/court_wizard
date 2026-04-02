use bevy::prelude::*;

use super::constants::*;
use crate::game::terrain::utils::preload_sprite_sheet;

/// Pre-loaded meshes and materials for boulder sprite variants.
#[derive(Resource)]
pub struct BoulderAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<StandardMaterial>; BOULDER_SPRITE_COUNT],
}

/// System to pre-load boulder sprite sheet and create one material per variant.
pub(super) fn preload_boulder_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (mesh, sprite_materials) = preload_sprite_sheet::<BOULDER_SPRITE_COUNT>(
        &mut meshes,
        &mut materials,
        &asset_server,
        "images/sprite_sheets/boulder_tiles.png",
        BOULDER_SPRITE_WIDTH,
        BOULDER_SPRITE_HEIGHT,
    );

    commands.insert_resource(BoulderAssets {
        mesh,
        materials: sprite_materials,
    });
}
