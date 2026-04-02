use bevy::prelude::*;

use super::constants::*;
use crate::game::terrain::utils::preload_sprite_sheet;

/// Pre-loaded meshes and materials for bush sprite variants.
#[derive(Resource)]
pub struct BushAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<StandardMaterial>; BUSH_SPRITE_COUNT],
}

/// System to pre-load bush sprite sheet and create one material per variant.
pub(super) fn preload_bush_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (mesh, sprite_materials) = preload_sprite_sheet::<BUSH_SPRITE_COUNT>(
        &mut meshes,
        &mut materials,
        &asset_server,
        "images/sprite_sheets/bush_tiles.png",
        BUSH_SPRITE_WIDTH,
        BUSH_SPRITE_HEIGHT,
    );

    commands.insert_resource(BushAssets {
        mesh,
        materials: sprite_materials,
    });
}
