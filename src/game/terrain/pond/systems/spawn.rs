use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::PondAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};

/// Spawns a single pond at the given position with the given radius.
pub(in crate::game) fn spawn_single_pond(
    commands: &mut Commands,
    assets: &PondAssets,
    x: f32,
    z: f32,
    radius: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let center = Vec3::new(x, 0.0, z);

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.material.clone()),
        Transform::from_xyz(x, POND_SURFACE_Y, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        Pond {
            center,
            radius,
            ripple_timer: 0.0,
        },
        OnGameplayScreen,
    ));

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::SlowTerrain(POND_FLOW_COST),
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });
}
