use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;

/// Sets up the battlefield and castle when entering the InGame state.
///
/// Spawns the battlefield ground plane, castle wall image, and point light in 3D space.
pub fn setup_battlefield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    castle_wall_assets: Res<CastleWallAssets>,
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
