//! Stuck-unit detection and perpendicular-nudge recovery.

use bevy::prelude::*;

use crate::game::components::Acceleration;

use super::super::components::{FlowFieldVelocity, StuckDetection};

/// Auto-inserts `StuckDetection` on entities that have `FlowFieldVelocity` but
/// don't yet have `StuckDetection`. This avoids modifying every spawn function.
pub fn init_stuck_detection(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<FlowFieldVelocity>, Without<StuckDetection>)>,
) {
    for (entity, transform) in &query {
        commands.entity(entity).insert(StuckDetection {
            last_check_pos: transform.translation,
            frames_since_check: 0,
            consecutive_stuck: 0,
        });
    }
}

/// Checks every 30 frames whether a unit has moved. After 3 consecutive stuck
/// checks (~1.5s), applies a perpendicular nudge force to break free.
pub fn detect_and_recover_stuck_units(
    mut query: Query<(
        &Transform,
        &FlowFieldVelocity,
        &mut StuckDetection,
        &mut Acceleration,
    )>,
) {
    const CHECK_INTERVAL: u32 = 30;
    const STUCK_THRESHOLD: f32 = 2.0;
    const STUCK_COUNT_FOR_NUDGE: u32 = 3;
    const NUDGE_FORCE: f32 = 400.0;

    for (transform, flow_vel, mut stuck, mut accel) in &mut query {
        stuck.frames_since_check += 1;
        if stuck.frames_since_check < CHECK_INTERVAL {
            continue;
        }
        stuck.frames_since_check = 0;

        // Skip units at destination or with no flow velocity
        if flow_vel.at_destination || flow_vel.velocity.length_squared() < 0.01 {
            stuck.consecutive_stuck = 0;
            stuck.last_check_pos = transform.translation;
            continue;
        }

        let distance_moved = transform.translation.distance(stuck.last_check_pos);

        if distance_moved < STUCK_THRESHOLD {
            stuck.consecutive_stuck += 1;

            if stuck.consecutive_stuck >= STUCK_COUNT_FOR_NUDGE {
                // Apply perpendicular nudge to flow velocity direction.
                // Alternate direction based on position hash for variety.
                let flow_dir = flow_vel.velocity.normalize_or_zero();
                let perp = Vec3::new(-flow_dir.z, 0.0, flow_dir.x);

                // Use position to determine nudge direction consistently
                let sign = if (transform.translation.x + transform.translation.z) as i32 % 2 == 0 {
                    1.0
                } else {
                    -1.0
                };

                accel.add_force(perp * NUDGE_FORCE * sign);
                stuck.consecutive_stuck = 0;
            }
        } else {
            stuck.consecutive_stuck = 0;
        }

        stuck.last_check_pos = transform.translation;
    }
}
