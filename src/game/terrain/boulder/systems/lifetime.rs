use bevy::prelude::*;

use super::super::components::Boulder;
use super::super::constants::*;
use crate::game::components::ObstacleHealth;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};

/// Ticks boulder lifetime and handles sinking animation.
pub fn tick_rock_lifetime(time: Res<Time>, mut rocks: Query<(&mut Boulder, &mut Transform)>) {
    let delta = time.delta_secs();

    for (mut rock, mut transform) in &mut rocks {
        rock.time_alive += delta;

        if rock.sinking {
            let sink_progress = ((rock.time_alive - (rock.sink_deadline - ROCK_SINK_DURATION))
                / ROCK_SINK_DURATION)
                .clamp(0.0, 1.0);
            // Sink from sprite position down to underground
            let base_y = BOULDER_SPRITE_HEIGHT / 2.0 - BOULDER_GROUND_CLIP;
            transform.translation.y = base_y * (1.0 - sink_progress);
        }
    }
}

pub fn cleanup_sunk_rocks(
    mut commands: Commands,
    rocks: Query<(Entity, &Boulder)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, rock) in &rocks {
        if rock.sinking && rock.time_alive >= rock.sink_deadline {
            let obs_bounds = rock.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(
                    Vec2::new(rock.center.x, rock.center.z),
                    rock.radius,
                )),
                rebuild: false,
            });

            commands.entity(entity).despawn();
        }
    }
}

pub fn destroy_dead_rocks(mut rocks: Query<(&mut Boulder, &ObstacleHealth)>) {
    for (mut rock, health) in &mut rocks {
        if health.is_dead() && !rock.sinking {
            rock.sinking = true;
            rock.sink_deadline = rock.time_alive + ROCK_SINK_DURATION;
        }
    }
}

/// Detects boulders whose Transform was moved externally (e.g. by teleport) and
/// updates the Boulder.center + pathfinding grid to match.
pub fn sync_teleported_rocks(
    mut rocks: Query<(&mut Boulder, &Transform), Changed<Transform>>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (mut rock, transform) in &mut rocks {
        let new_center = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
        if (rock.center - new_center).length_squared() < 0.01 {
            continue;
        }

        // Remove obstacle at old position
        let old_bounds = rock.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(old_bounds[0], old_bounds[1], old_bounds[2], old_bounds[3]),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(
                Vec2::new(rock.center.x, rock.center.z),
                rock.radius,
            )),
            rebuild: false,
        });

        // Update center
        rock.center = new_center;

        // Add obstacle at new position
        let new_bounds = rock.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(new_bounds[0], new_bounds[1], new_bounds[2], new_bounds[3]),
            obstacle_type: ObstacleType::Blocked,
            shape: Some(ObstacleShape::circle(
                Vec2::new(rock.center.x, rock.center.z),
                rock.radius,
            )),
            rebuild: false,
        });
    }
}
