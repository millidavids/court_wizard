use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;

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
    let battlefield_mesh = Plane3d::default().mesh().size(3000.0, 3000.0);

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

    // Spawn castle as a raised platform (Plane3d) above the battlefield
    let castle_plane = Plane3d::default().mesh().size(400.0, 300.0);

    commands.spawn((
        Mesh3d(meshes.add(castle_plane)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: CASTLE_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(-1100.0, 500.0, 1100.0), // Bottom-left corner, raised high above ground
        Castle,
        OnGameplayScreen,
    ));
}
