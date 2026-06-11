use bevy::prelude::*;

use crate::game::components::{Acceleration, Velocity};
use crate::game::units::archer::Archer;
use crate::game::units::components::{Corpse, Flying, Hitbox, TargetingVelocity};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Zeroes out targeting velocity when a wall blocks the line to the target.
///
/// Without this, units behind a wall push forward (via flocking/separation)
/// because their targeting still points straight through the wall.  By
/// suppressing targeting when blocked, those units follow the flow field
/// around the wall instead.
pub fn suppress_targeting_through_walls(
    walls: Query<&WallOfStone>,
    mut units: Query<
        (&Transform, &mut TargetingVelocity),
        (Without<Corpse>, Without<Archer>, Without<Flying>),
    >,
) {
    for (transform, mut targeting) in &mut units {
        let dir = targeting.velocity;
        if dir == Vec3::ZERO {
            continue;
        }

        // Project from unit position along targeting direction up to the distance to target
        let check_dist = targeting.distance_to_target.max(1.0);
        let end = transform.translation + dir * check_dist;

        for wall in &walls {
            if wall
                .line_segment_intersects(transform.translation, end)
                .is_some()
            {
                targeting.velocity = Vec3::ZERO;
                break;
            }
        }
    }
}

/// Applies a steering force to units approaching walls so they navigate around them,
/// and a proximity-based repulsion force to push units away from nearby walls.
pub fn apply_wall_avoidance(
    walls: Query<&WallOfStone>,
    mut units: Query<
        (&Transform, &Velocity, &mut Acceleration, &Hitbox),
        (Without<Corpse>, Without<Flying>),
    >,
) {
    const AVOIDANCE_DISTANCE: f32 = 80.0; // How far ahead units look for walls
    const AVOIDANCE_FORCE: f32 = 800.0; // Strength of the avoidance steering
    const REPULSION_MARGIN: f32 = 15.0; // Extra distance around wall for repulsion zone
    const REPULSION_FORCE_MAX: f32 = 600.0; // Maximum repulsion force at wall surface

    for (transform, velocity, mut acceleration, hitbox) in &mut units {
        let vel = Vec3::new(velocity.x, 0.0, velocity.z);
        let speed = vel.length();

        for wall in &walls {
            // --- Look-ahead avoidance (velocity-based) ---
            if speed >= 1.0 {
                let vel_dir = vel / speed;
                let look_ahead = transform.translation + vel_dir * AVOIDANCE_DISTANCE;

                let diff = Vec3::new(
                    look_ahead.x - wall.center.x,
                    0.0,
                    look_ahead.z - wall.center.z,
                );
                let forward_proj = diff.dot(wall.forward);
                let right_proj = diff.dot(wall.right);

                let forward_pen = wall.half_length + hitbox.radius - forward_proj.abs();
                let right_pen = wall.half_width + hitbox.radius - right_proj.abs();

                if forward_pen > 0.0 && right_pen > 0.0 {
                    let steer = if forward_pen < right_pen {
                        wall.right * right_proj.signum()
                    } else {
                        wall.forward * forward_proj.signum()
                    };
                    let proximity =
                        1.0 - (forward_pen.min(right_pen) / AVOIDANCE_DISTANCE).min(1.0);
                    acceleration.add_force(steer * AVOIDANCE_FORCE * proximity);
                }
            }

            // --- Proximity-based repulsion (position-based) ---
            let diff = Vec3::new(
                transform.translation.x - wall.center.x,
                0.0,
                transform.translation.z - wall.center.z,
            );
            let forward_proj = diff.dot(wall.forward);
            let right_proj = diff.dot(wall.right);

            let expanded_half_fwd = wall.half_length + hitbox.radius + REPULSION_MARGIN;
            let expanded_half_right = wall.half_width + hitbox.radius + REPULSION_MARGIN;

            let fwd_pen = expanded_half_fwd - forward_proj.abs();
            let right_pen = expanded_half_right - right_proj.abs();

            if fwd_pen > 0.0 && right_pen > 0.0 {
                // Inside repulsion zone — push along the axis of least penetration
                let (push_dir, penetration, margin) = if fwd_pen < right_pen {
                    (
                        wall.forward * forward_proj.signum(),
                        fwd_pen,
                        expanded_half_fwd,
                    )
                } else {
                    (
                        wall.right * right_proj.signum(),
                        right_pen,
                        expanded_half_right,
                    )
                };

                // Linear falloff: strongest at wall surface, zero at margin edge
                let strength = (penetration / margin).min(1.0);
                acceleration.add_force(push_dir * REPULSION_FORCE_MAX * strength);
            }
        }
    }
}

/// Applies a circular push-out correction to a unit's transform.
///
/// Pushes units out of any active Wall of Stone or Boulder entities.
/// Runs after movement systems to ensure units cannot walk through walls or rocks.
fn apply_circular_push_out(
    corrected: Option<Vec3>,
    transform: &mut Mut<Transform>,
    total_correction: &mut Vec3,
    corrected_this_pass: &mut bool,
    had_collision: &mut bool,
) {
    if let Some(corrected) = corrected {
        let correction = Vec3::new(
            corrected.x - transform.translation.x,
            0.0,
            corrected.z - transform.translation.z,
        );
        transform.translation.x = corrected.x;
        transform.translation.z = corrected.z;
        *total_correction += correction;
        *corrected_this_pass = true;
        *had_collision = true;
    }
}

/// Uses multiple iterations so that obstacle intersections are resolved correctly —
/// being pushed out of obstacle A won't leave the unit stuck in obstacle B.
pub fn enforce_wall_collision(
    walls: Query<&WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees: Query<&crate::game::terrain::tree::components::Tree>,
    mut units: Query<
        (
            &mut Transform,
            &Hitbox,
            Option<&mut Velocity>,
            Option<&TargetingVelocity>,
        ),
        (Without<Corpse>, Without<Flying>),
    >,
) {
    const MAX_ITERATIONS: u32 = 4;

    for (mut transform, hitbox, mut velocity_opt, targeting_velocity) in &mut units {
        // Get the desired movement direction for intelligent collision response
        let desired_direction = if let Some(vel) = velocity_opt.as_ref() {
            Some(Vec3::new(vel.x, 0.0, vel.z).normalize_or_zero())
        } else {
            targeting_velocity.map(|tv| tv.velocity.normalize_or_zero())
        };

        // Accumulate total correction across all iterations for velocity adjustment
        let mut total_correction = Vec3::ZERO;
        let mut had_collision = false;

        // Iterate multiple times to resolve intersections where pushing out of
        // one obstacle lands the unit inside another
        for _ in 0..MAX_ITERATIONS {
            let mut corrected_this_pass = false;

            for wall in &walls {
                if let Some(corrected) =
                    wall.push_out(transform.translation, hitbox.radius, desired_direction)
                {
                    let correction = Vec3::new(
                        corrected.x - transform.translation.x,
                        0.0,
                        corrected.z - transform.translation.z,
                    );

                    transform.translation.x = corrected.x;
                    transform.translation.z = corrected.z;
                    total_correction += correction;
                    corrected_this_pass = true;
                    had_collision = true;
                }
            }

            for rock in &rocks {
                if rock.sinking {
                    continue;
                }
                apply_circular_push_out(
                    rock.push_out(transform.translation, hitbox.radius),
                    &mut transform,
                    &mut total_correction,
                    &mut corrected_this_pass,
                    &mut had_collision,
                );
            }

            for tree in &trees {
                apply_circular_push_out(
                    tree.push_out(transform.translation, hitbox.radius),
                    &mut transform,
                    &mut total_correction,
                    &mut corrected_this_pass,
                    &mut had_collision,
                );
            }

            // No more overlaps — stable position found
            if !corrected_this_pass {
                break;
            }
        }

        // Adjust velocity once based on accumulated correction
        if had_collision && let Some(ref mut velocity) = velocity_opt {
            let correction_normal = total_correction.normalize_or_zero();
            let velocity_vec = Vec3::new(velocity.x, 0.0, velocity.z);
            let velocity_magnitude = velocity_vec.length();

            // Remove velocity component going into obstacles, keep tangential
            let perpendicular_component = velocity_vec.dot(correction_normal);
            if perpendicular_component < 0.0 {
                let tangent_velocity = velocity_vec - correction_normal * perpendicular_component;

                let correction_magnitude = total_correction.length();
                let repulsion_strength = (correction_magnitude / hitbox.radius).min(1.0);
                let repulsion_force =
                    correction_normal * velocity_magnitude * repulsion_strength * 1.5;

                let final_velocity = tangent_velocity + repulsion_force;
                let final_velocity_normalized = final_velocity.normalize_or_zero();
                velocity.x = final_velocity_normalized.x * velocity_magnitude;
                velocity.z = final_velocity_normalized.z * velocity_magnitude;
            }
        }
    }
}
