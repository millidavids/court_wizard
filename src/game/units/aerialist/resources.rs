use bevy::prelude::*;

use crate::game::units::constants::{DEFAULT_SPRITE_HEIGHT, DEFAULT_SPRITE_WIDTH};

/// Pre-loaded meshes and materials for aerialist units.
#[derive(Resource)]
pub struct AerialistAssets {
    pub sprite_mesh: Handle<Mesh>,
    pub sprite_texture: Handle<Image>,
    pub death_texture: Handle<Image>,
}

pub(super) fn preload_aerialist_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let sprite_texture = asset_server.load("images/sprite_sheets/aerialist-flying_2-frames.png");
    let death_texture = asset_server.load("images/sprite_sheets/aerialist-death_6-frames.png");

    let assets = AerialistAssets {
        sprite_mesh: meshes.add(Rectangle::new(DEFAULT_SPRITE_WIDTH, DEFAULT_SPRITE_HEIGHT)),
        sprite_texture,
        death_texture,
    };

    commands.insert_resource(assets);
}
