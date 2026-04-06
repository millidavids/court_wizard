use bevy::prelude::*;

use super::constants::*;
use crate::game::terrain::utils::preload_wind_sway_sprite_sheet;
use crate::game::terrain::wind_sway::constants::*;
use crate::game::terrain::wind_sway::material::WindSwayMaterial;

/// Pre-loaded meshes and materials for bush sprite variants.
#[derive(Resource)]
pub struct BushAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<WindSwayMaterial>; BUSH_SPRITE_COUNT],
}

/// System to pre-load bush sprite sheet and create one material per variant.
pub(super) fn preload_bush_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WindSwayMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (mesh, sprite_materials) = preload_wind_sway_sprite_sheet::<BUSH_SPRITE_COUNT>(
        &mut meshes,
        &mut materials,
        &asset_server,
        "images/sprite_sheets/bush_tiles.png",
        BUSH_SPRITE_WIDTH,
        BUSH_SPRITE_HEIGHT,
        BUSH_SWAY_AMPLITUDE,
        BUSH_SWAY_SPEED,
    );

    commands.insert_resource(BushAssets {
        mesh,
        materials: sprite_materials,
    });
}
