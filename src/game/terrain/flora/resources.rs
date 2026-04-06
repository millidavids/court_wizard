use bevy::prelude::*;

use super::constants::{FLORA_SPRITE_COUNT, FLORA_SPRITE_HEIGHT, FLORA_SPRITE_WIDTH};
use crate::game::terrain::utils::preload_wind_sway_sprite_sheet;
use crate::game::terrain::wind_sway::constants::*;
use crate::game::terrain::wind_sway::material::WindSwayMaterial;

/// Pre-loaded meshes and materials for flora sprites.
#[derive(Resource)]
pub struct FloraAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<WindSwayMaterial>; FLORA_SPRITE_COUNT],
}

/// Pre-loads the flora sprite sheet and creates one material per sprite variant.
pub(super) fn preload_flora_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WindSwayMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (mesh, sprite_materials) = preload_wind_sway_sprite_sheet::<FLORA_SPRITE_COUNT>(
        &mut meshes,
        &mut materials,
        &asset_server,
        "images/sprite_sheets/flora_tiles.png",
        FLORA_SPRITE_WIDTH,
        FLORA_SPRITE_HEIGHT,
        FLORA_SWAY_AMPLITUDE,
        FLORA_SWAY_SPEED,
    );

    commands.insert_resource(FloraAssets {
        mesh,
        materials: sprite_materials,
    });
}
