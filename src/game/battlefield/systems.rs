use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;

// ===== Right Wall Constants =====

/// The right wall image is 320x180 (16:9 aspect ratio).
const RIGHT_WALL_IMAGE_WIDTH: f32 = 320.0;
const RIGHT_WALL_IMAGE_HEIGHT: f32 = 180.0;

/// Width of the right wall backdrop in world units.
const RIGHT_WALL_WIDTH: f32 = 4000.0;

/// Height derived from aspect ratio.
const RIGHT_WALL_HEIGHT: f32 = RIGHT_WALL_WIDTH * (RIGHT_WALL_IMAGE_HEIGHT / RIGHT_WALL_IMAGE_WIDTH);

/// Position of the right wall (along the +X edge of the battlefield, facing inward).
/// Centered along the Z axis, raised so it fills the background.
/// Positioned so the bottom-left corner of the image meets the top corner of the battlefield.
/// Bottom edge at Y=0 (center Y = half height), far end at Z = -BATTLEFIELD_HALF
/// (center Z = -BATTLEFIELD_HALF + half width).
const BATTLEFIELD_HALF: f32 = BATTLEFIELD_SIZE / 2.0;
const RIGHT_WALL_POSITION: Vec3 = Vec3::new(
    BATTLEFIELD_HALF,
    RIGHT_WALL_HEIGHT / 2.0,
    -BATTLEFIELD_HALF + RIGHT_WALL_WIDTH / 2.0,
);

/// Rotation so the wall faces inward (toward -X).
/// A Rectangle mesh faces +Z by default, so rotate 90° to face -X.
const RIGHT_WALL_ROTATION_DEGREES: f32 = 90.0;

/// Sets up the battlefield and castle when entering the InGame state.
///
/// Spawns the battlefield ground plane, castle wall image, and point light in 3D space.
pub fn setup_battlefield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    castle_wall_assets: Res<CastleWallAssets>,
    right_wall_assets: Res<RightWallAssets>,
) {
    // Add a light source so we can see 3D objects
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1000.0, 0.0),
        OnGameplayScreen,
    ));

    // Spawn battlefield as ground plane at origin
    let battlefield_mesh = Plane3d::default()
        .mesh()
        .size(BATTLEFIELD_SIZE, BATTLEFIELD_SIZE);

    commands.spawn((
        Mesh3d(meshes.add(battlefield_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: BATTLEFIELD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0), // Centered at origin
        Battlefield,
        OnGameplayScreen,
    ));

    // Spawn castle wall as a textured plane the wizard stands on
    spawn_castle_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        &castle_wall_assets,
        CASTLE_POSITION,
        CASTLE_ROTATION_DEGREES,
        OnGameplayScreen,
    );

    // Spawn right wall backdrop (vertical plane at the right edge)
    let right_wall_mesh = Rectangle::new(RIGHT_WALL_WIDTH, RIGHT_WALL_HEIGHT);
    let right_wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(right_wall_assets.texture.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(right_wall_mesh)),
        MeshMaterial3d(right_wall_material),
        Transform::from_translation(RIGHT_WALL_POSITION)
            .with_rotation(Quat::from_rotation_y(RIGHT_WALL_ROTATION_DEGREES.to_radians())),
        RightWall,
        OnGameplayScreen,
    ));
}

/// Spawns the castle wall as a textured plane at the given position.
///
/// The plane uses the castle_wall.png image at its natural aspect ratio,
/// scaled to CASTLE_WIDTH wide. The wizard and cauldron stand on this plane.
pub fn spawn_castle_wall<M: Component + Clone>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    castle_wall_assets: &CastleWallAssets,
    castle_position: Vec3,
    rotation_degrees: f32,
    screen_marker: M,
) {
    // Size the plane to match the image's aspect ratio (629x1024), scaled 3x.
    // Width = CASTLE_WIDTH * 3, depth computed from aspect ratio.
    const IMAGE_WIDTH: f32 = 629.0;
    const IMAGE_HEIGHT: f32 = 1024.0;
    let plane_width = CASTLE_WIDTH * 3.0;
    let plane_depth = plane_width * (IMAGE_HEIGHT / IMAGE_WIDTH);

    let wall_mesh = Plane3d::default()
        .mesh()
        .size(plane_width, plane_depth);

    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(castle_wall_assets.texture.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(wall_mesh)),
        MeshMaterial3d(wall_material),
        Transform::from_translation(castle_position)
            .with_rotation(Quat::from_rotation_y(rotation_degrees.to_radians())),
        Castle,
        screen_marker,
    ));
}
