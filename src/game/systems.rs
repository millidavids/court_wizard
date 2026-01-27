use bevy::prelude::*;

use super::components::Billboard;

/// Updates billboard entities to always face the camera.
///
/// Rotates entities with the Billboard component around the Y axis so they remain
/// perpendicular to the camera's forward direction on the XZ plane.
pub fn update_billboards(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut billboard_query: Query<&mut Transform, (With<Billboard>, Without<Camera3d>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Get camera's forward direction on XZ plane
    let camera_forward = camera_transform.forward();
    let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize();

    // Calculate rotation to face camera (rotate around Y axis)
    // We want the billboard's local -Z axis to point toward the camera
    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, camera_forward_xz);

    // Apply rotation to all billboards
    for mut transform in &mut billboard_query {
        // Keep the existing position and scale, only update rotation
        transform.rotation = rotation;
    }
}
