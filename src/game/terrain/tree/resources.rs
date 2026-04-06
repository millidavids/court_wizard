use bevy::prelude::*;

use super::constants::*;
use crate::game::terrain::utils::preload_wind_sway_sprite_sheet;
use crate::game::terrain::wind_sway::constants::*;
use crate::game::terrain::wind_sway::material::WindSwayMaterial;

/// Pre-loaded meshes and materials for trees.
#[derive(Resource)]
pub struct TreeAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<WindSwayMaterial>; TREE_SPRITE_COUNT],
}

/// System to pre-load tree assets at startup.
pub(super) fn preload_tree_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WindSwayMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (mesh, sprite_materials) = preload_wind_sway_sprite_sheet::<TREE_SPRITE_COUNT>(
        &mut meshes,
        &mut materials,
        &asset_server,
        "images/sprite_sheets/tree_tiles.png",
        TREE_SPRITE_WIDTH,
        TREE_SPRITE_HEIGHT,
        WIND_SWAY_AMPLITUDE,
        WIND_SWAY_SPEED,
    );

    commands.insert_resource(TreeAssets {
        mesh,
        materials: sprite_materials,
    });
}
