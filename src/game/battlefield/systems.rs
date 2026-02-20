use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;

/// Sets up the battlefield and castle when entering the InGame state.
///
/// Spawns the battlefield ground plane, castle platform, and point light in 3D space.
pub fn setup_battlefield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // Spawn castle as a 3D box extending from the top surface down to battlefield level
    let castle_box = Cuboid::new(CASTLE_WIDTH, CASTLE_HEIGHT, CASTLE_DEPTH);
    // CASTLE_POSITION is the top surface; shift down by half height to center the box
    let castle_center = CASTLE_POSITION - Vec3::new(0.0, CASTLE_HEIGHT / 2.0, 0.0);

    commands.spawn((
        Mesh3d(meshes.add(castle_box)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: CASTLE_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(castle_center)
            .with_rotation(Quat::from_rotation_y(CASTLE_ROTATION_DEGREES.to_radians())),
        Castle,
        OnGameplayScreen,
    ));
}
