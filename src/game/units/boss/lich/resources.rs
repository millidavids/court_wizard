use bevy::prelude::*;

use super::constants::*;
use crate::game::units::systems::create_sprite_material;

/// Pre-loaded mesh and materials for the Lich boss.
#[derive(Resource)]
pub struct LichAssets {
    pub mesh: Handle<Mesh>,
    pub floating_material: Handle<StandardMaterial>,
    pub casting_material: Handle<StandardMaterial>,
}

/// System to pre-load Lich assets at startup.
pub(super) fn preload_lich_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floating_texture =
        asset_server.load("images/sprite_sheets/lich-floating_4-frames.png");
    let casting_texture =
        asset_server.load("images/sprite_sheets/lich-casting_4-frames.png");

    let mesh = meshes.add(Rectangle::new(LICH_SPRITE_WIDTH, LICH_SPRITE_HEIGHT));

    let floating_material = create_sprite_material(
        &mut materials,
        floating_texture,
        Color::WHITE,
        LICH_FRAME_UV,
        Vec2::ZERO,
    );
    let casting_material = create_sprite_material(
        &mut materials,
        casting_texture,
        Color::WHITE,
        LICH_FRAME_UV,
        Vec2::ZERO,
    );

    commands.insert_resource(LichAssets {
        mesh,
        floating_material,
        casting_material,
    });
}
