use bevy::prelude::*;

use super::material::WindSwayMaterial;

/// Propagates elapsed time to all vegetation materials for synchronized sway.
pub(super) fn update_wind_sway_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<WindSwayMaterial>>,
) {
    let t = time.elapsed_secs();
    for (_id, material) in materials.iter_mut() {
        material.sway_params.x = t;
    }
}
