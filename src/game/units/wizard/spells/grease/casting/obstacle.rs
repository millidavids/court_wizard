use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use bevy::prelude::*;

/// Helper to write an obstacle event for a grease zone.
pub(crate) fn write_grease_obstacle(
    origin: Vec3,
    radius: f32,
    obstacle_type: ObstacleType,
    events: &mut MessageWriter<ObstacleChanged>,
) {
    let origin_2d = Vec2::new(origin.x, origin.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type,
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        rebuild: false,
    });
}
