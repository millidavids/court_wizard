use bevy::prelude::*;

use super::constants::{FLORA_SPRITE_COUNT, FLORA_SPRITE_HEIGHT, FLORA_SPRITE_WIDTH};
use crate::game::units::systems::create_sprite_material;

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
    let texture: Handle<Image> = asset_server.load("images/sprite_sheets/flora_tiles.png");
    let mesh = meshes.add(Rectangle::new(FLORA_SPRITE_WIDTH, FLORA_SPRITE_HEIGHT));

    let u_width = 1.0 / FLORA_SPRITE_COUNT as f32;
    let sprite_materials: [Handle<StandardMaterial>; FLORA_SPRITE_COUNT] =
        std::array::from_fn(|i| {
            let u_offset = i as f32 * u_width;
            create_sprite_material(
                &mut materials,
                texture.clone(),
                Color::WHITE,
                Vec2::new(u_width, 1.0),
                Vec2::new(u_offset, 0.0),
            )
        });

    commands.insert_resource(FloraAssets {
        mesh,
        materials: sprite_materials,
    });
}
