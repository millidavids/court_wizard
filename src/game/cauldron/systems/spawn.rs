use bevy::math::Affine2;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use super::super::resources::CauldronAssets;
use crate::game::components::{Billboard, OnGameplayScreen};

/// Loads the cauldron sprite sheet texture.
pub fn load_cauldron_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite_texture = asset_server.load("images/cauldron-64px-9.png");
    commands.insert_resource(CauldronAssets { sprite_texture });
}

/// Spawns the cauldron entity as an animated sprite billboard.
pub fn spawn_cauldron(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cauldron_assets: &CauldronAssets,
    // World position for the cauldron. Single-player and the multiplayer HOST use
    // `CAULDRON_POSITION`; the multiplayer GUEST uses `CAULDRON_2_POSITION` so the
    // cauldron sits beside the guest's own (mirrored) wizard.
    position: Vec3,
) {
    // Create a quad mesh for the billboard
    let quad_mesh = Rectangle::new(
        constants::CAULDRON_SPRITE_SIZE,
        constants::CAULDRON_SPRITE_SIZE,
    );

    // UV transform for first frame: scale to 1/3 to show only top-left frame
    let grid_size = constants::CAULDRON_SPRITE_GRID_SIZE as f32;
    let frame_scale = 1.0 / grid_size;
    let uv_transform = Affine2::from_scale(Vec2::splat(frame_scale));

    // Create material with sprite sheet texture
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(cauldron_assets.sprite_texture.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        uv_transform,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        Cauldron,
        CauldronState::default(),
        CauldronAnimation::new(),
        Billboard,
        OnGameplayScreen,
    ));
}
