use super::super::components::{DispelledWall, WallOfStone};
use super::super::constants::*;
use crate::game::pathfinding::{ObstacleChanged, ObstacleShape, ObstacleType};
use bevy::prelude::*;

/// Advances wall lifetime and triggers sinking phase (skips permanent walls).
pub fn tick_wall_lifetime(time: Res<Time>, mut walls: Query<&mut WallOfStone>) {
    let delta = time.delta_secs();
    for mut wall in &mut walls {
        if wall.permanent {
            continue;
        }
        wall.time_alive += delta;
        if !wall.sinking && wall.time_alive >= wall.duration - WALL_SINK_DURATION {
            wall.sinking = true;
        }
    }
}

/// Animates walls sinking into the ground during their final seconds.
pub fn animate_sinking_walls(mut walls: Query<(&WallOfStone, &mut Transform)>) {
    for (wall, mut transform) in &mut walls {
        if wall.sinking {
            let sink_elapsed = wall.time_alive - (wall.duration - WALL_SINK_DURATION);
            let sink_progress = (sink_elapsed / WALL_SINK_DURATION).clamp(0.0, 1.0);
            let target_y = wall.height / 2.0 - wall.height * sink_progress;
            transform.translation.y = target_y;
        }
    }
}

/// Despawns walls that have exceeded their duration (skips permanent walls).
pub fn cleanup_expired_walls(
    mut commands: Commands,
    walls: Query<(Entity, &WallOfStone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
) {
    for (entity, wall) in &walls {
        if wall.permanent {
            continue;
        }
        if wall.time_alive >= wall.duration {
            commands.entity(entity).try_despawn();

            // Notify pathfinding system that the obstacle is removed
            let obs_bounds = wall.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::obb_from_center(
                    wall.center,
                    wall.forward,
                    wall.half_length,
                    wall.half_width,
                )),
                rebuild: false,
            });

            // Notify remote peer to update their pathfinding grid
            if let Some(ref mut conn) = connection {
                conn.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::WallPlaced {
                        bounds: obs_bounds,
                        placed: false,
                    },
                );
            }
        }
    }
}

/// Processes walls marked for dispel — starts the sinking animation.
pub fn handle_dispelled_walls(
    mut commands: Commands,
    mut walls: Query<(Entity, &mut WallOfStone, &DispelledWall)>,
) {
    for (entity, mut wall, dispelled) in &mut walls {
        if !wall.sinking {
            wall.sinking = true;
            wall.permanent = false;
            wall.duration = wall.time_alive + dispelled.sink_duration;
        }
        commands.entity(entity).remove::<DispelledWall>();
    }
}

/// Destroys walls that have lost all HP by triggering the existing sink + cleanup pipeline.
pub fn destroy_dead_walls(
    mut walls: Query<(&mut WallOfStone, &super::super::components::WallHealth)>,
) {
    for (mut wall, wall_health) in &mut walls {
        if wall_health.is_dead() && !wall.sinking {
            // Enter sinking phase — existing tick_wall_lifetime + cleanup_expired_walls
            // will handle the rest (obstacle removal, despawn, network sync).
            wall.sinking = true;
            wall.permanent = false;
            wall.duration = wall.time_alive + WALL_SINK_DURATION;
        }
    }
}
