use bevy::math::Affine2;
use bevy::prelude::*;

use super::constants::{
    CHARM_BEAM_COLOR, DISINTEGRATE_BEAM_COLOR, FEAR_BEAM_COLOR, PETRIFY_BEAM_COLOR,
    RAY_BODY_SPRITE_SIZE, RAY_EYE_SPRITE_SIZE, RAY_STALK_PARTICLE_RADIUS, TELEPORT_BEAM_COLOR,
};
use crate::game::units::boss::utils::EYE_FRAME_UV;

#[derive(Resource)]
pub struct RayAssets {
    pub body_mesh: Handle<Mesh>,
    /// Quad mesh for eye sprites (Ray's 5 boss eyes).
    pub eye_sprite_mesh: Handle<Mesh>,
    pub particle_mesh: Handle<Mesh>,
    pub particle_material: Handle<StandardMaterial>,
    pub body_material: Handle<StandardMaterial>,
    pub eye_materials: [Handle<StandardMaterial>; 5],
    pub eye_inactive_material: Handle<StandardMaterial>,
    pub beam_materials: [Handle<StandardMaterial>; 5],
}

pub(super) fn preload_ray_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let eye_texture: Handle<Image> =
        asset_server.load("images/sprite_sheets/eye-pulsing_4-frames.png");
    let heart_texture: Handle<Image> =
        asset_server.load("images/sprite_sheets/ray-beating_4-frames.png");
    let eye_uv_transform = Affine2::from_scale_angle_translation(EYE_FRAME_UV, 0.0, Vec2::ZERO);
    let beam_colors = [
        PETRIFY_BEAM_COLOR,
        DISINTEGRATE_BEAM_COLOR,
        FEAR_BEAM_COLOR,
        CHARM_BEAM_COLOR,
        TELEPORT_BEAM_COLOR,
    ];

    let beam_mats: [Handle<StandardMaterial>; 5] = std::array::from_fn(|i| {
        let c = beam_colors[i].to_srgba();
        materials.add(StandardMaterial {
            base_color: beam_colors[i],
            emissive: LinearRgba::new(c.red * 3.0, c.green * 3.0, c.blue * 3.0, 1.0),
            unlit: true,
            cull_mode: None,
            ..default()
        })
    });

    let assets = RayAssets {
        body_mesh: meshes.add(Rectangle::new(RAY_BODY_SPRITE_SIZE, RAY_BODY_SPRITE_SIZE)),
        eye_sprite_mesh: meshes.add(Rectangle::new(RAY_EYE_SPRITE_SIZE, RAY_EYE_SPRITE_SIZE)),
        particle_mesh: meshes.add(Sphere::new(RAY_STALK_PARTICLE_RADIUS)),
        particle_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 1.0, 0.9, 0.4),
            emissive: LinearRgba::new(2.0, 3.0, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        body_material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(heart_texture),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            cull_mode: None,
            uv_transform: eye_uv_transform,
            ..default()
        }),
        eye_materials: {
            let eye_hues = [
                Color::srgb(0.85, 0.85, 0.78), // Petrification — warm stone gray
                Color::srgb(1.0, 0.55, 0.25),  // Disintegration — vivid orange
                Color::srgb(0.75, 0.40, 1.0),  // Fear — vivid purple
                Color::srgb(1.0, 0.45, 0.75),  // MindControl — vivid pink
                Color::srgb(0.30, 0.90, 1.0),  // Teleportation — vivid cyan
            ];
            std::array::from_fn(|i| {
                let c = eye_hues[i].to_srgba();
                materials.add(StandardMaterial {
                    base_color: eye_hues[i],
                    base_color_texture: Some(eye_texture.clone()),
                    emissive: LinearRgba::new(c.red * 2.5, c.green * 2.5, c.blue * 2.5, 1.0),
                    alpha_mode: AlphaMode::Mask(0.5),
                    unlit: true,
                    cull_mode: None,
                    uv_transform: eye_uv_transform,
                    ..default()
                })
            })
        },
        eye_inactive_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.3, 0.3, 0.3, 0.5),
            base_color_texture: Some(eye_texture.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            uv_transform: eye_uv_transform,
            ..default()
        }),
        beam_materials: beam_mats,
    };

    commands.insert_resource(assets);
}
