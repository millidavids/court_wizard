use bevy::prelude::*;

use crate::game::constants::KING_CORPSE_COLOR;
use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
use crate::game::units::systems::create_corpse_sprite_materials;

use super::constants::*;

/// Pre-loaded meshes and materials for the king unit.
#[derive(Resource)]
pub struct KingAssets {
    /// Rectangle mesh for sprite rendering.
    pub sprite_mesh: Handle<Mesh>,
    /// Combined sprite sheet texture. Reuses infantry textures.
    pub sprite_texture: Handle<Image>,
    pub corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// System to pre-load king assets at startup.
pub(super) fn preload_king_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // King reuses infantry sprite texture
    let sprite_texture =
        asset_server.load("images/sprite_sheets/infantry-walking_8-frames.png");

    let corpse_materials =
        create_corpse_sprite_materials(&mut materials, sprite_texture.clone(), KING_CORPSE_COLOR);

    let assets = KingAssets {
        sprite_mesh: meshes.add(Rectangle::new(KING_SPRITE_WIDTH, KING_SPRITE_HEIGHT)),
        sprite_texture,
        corpse_materials,
    };

    commands.insert_resource(assets);
}
