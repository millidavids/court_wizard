use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;

/// Sets up the wizard when entering the InGame state.
///
/// Spawns the wizard entity as a triangle on the castle platform in 3D space.
pub fn setup_wizard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn wizard as a triangle on the raised castle platform
    let wizard_size = 60.0;
    let wizard_triangle = Triangle2d::new(
        Vec2::new(0.0, wizard_size / 2.0),                 // Top vertex
        Vec2::new(-wizard_size / 2.0, -wizard_size / 2.0), // Bottom-left
        Vec2::new(wizard_size / 2.0, -wizard_size / 2.0),  // Bottom-right
    );

    commands.spawn((
        Mesh3d(meshes.add(wizard_triangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WIZARD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(-1150.0, 1230.0, 1400.0), // On castle platform, slightly above it
        Wizard,
        OnGameplayScreen,
    ));
}
