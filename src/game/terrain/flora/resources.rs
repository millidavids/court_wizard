use bevy::prelude::*;

use super::constants::{FLORA_SPRITE_COUNT, FLORA_SPRITE_HEIGHT, FLORA_SPRITE_WIDTH};
use crate::game::terrain::utils::preload_sprite_sheet;

/// Pre-loaded meshes and materials for flora sprites.
#[derive(Resource)]
pub struct FloraAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<StandardMaterial>; FLORA_SPRITE_COUNT],
}

/// Pre-loads the flora sprite sheet and creates one material per sprite variant.
pub(super) fn preload_flora_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (mesh, sprite_materials) = preload_sprite_sheet::<FLORA_SPRITE_COUNT>(
        &mut meshes,
        &mut materials,
        &asset_server,
        "images/sprite_sheets/flora_tiles.png",
        FLORA_SPRITE_WIDTH,
        FLORA_SPRITE_HEIGHT,
    );

    commands.insert_resource(FloraAssets {
        mesh,
        materials: sprite_materials,
    });
}
