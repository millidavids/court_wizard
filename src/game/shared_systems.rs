use bevy::prelude::*;

use super::components::Velocity;

/// Moves units based on their velocity.
///
/// Applies 3D velocity to entity positions using delta time for frame-rate independence.
/// Units automatically scale with perspective as they move along the Z axis.
pub fn move_units(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
        transform.translation.z += velocity.z * time.delta_secs();
    }
}

/// Cleans up all game entities when exiting the InGame state.
pub fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<super::components::OnGameplayScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
