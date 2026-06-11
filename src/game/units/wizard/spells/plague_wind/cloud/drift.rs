use super::super::components::PlagueWindCloud;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use bevy::prelude::*;

/// Moves the plague wind cloud in its drift direction and updates pathfinding.
pub fn move_plague_wind_cloud(
    time: Res<Time>,
    // Host-only — the guest mirrors cloud position via the snapshot, so the
    // ghost cloud must NOT independently drift (would diverge from host AND
    // double-update the pathfinding grid).
    mut clouds: Query<
        (&mut PlagueWindCloud, &mut Transform),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (mut cloud, mut transform) in clouds.iter_mut() {
        // Remove old pathfinding bounds
        let old_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        let buffered = cloud.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(old_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(old_origin_2d, buffered)),
            rebuild: false,
        });

        // Move cloud
        let movement = cloud.direction * cloud.speed * delta;
        cloud.origin += movement;
        transform.translation.x = cloud.origin.x;
        transform.translation.z = cloud.origin.z;

        // Add new pathfinding bounds
        let new_origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(new_origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Hazard(10.0),
            shape: Some(ObstacleShape::circle(new_origin_2d, buffered)),
            rebuild: false,
        });
    }
}

/// Cleans up expired plague wind clouds and notifies pathfinding.
pub fn cleanup_plague_wind_cloud(
    mut commands: Commands,
    // Ghost clouds are reconciliation-driven and host-authoritative; never run
    // the lifetime cleanup (which fires `ObstacleChanged` into the pathfinding
    // grid) on them. Matches the same exclusion on `move_plague_wind_cloud`.
    clouds: Query<
        (Entity, &PlagueWindCloud),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, cloud) in &clouds {
        if cloud.time_alive >= cloud.duration {
            let origin_2d = Vec2::new(cloud.origin.x, cloud.origin.z);
            let buffered = cloud.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}
