use bevy::prelude::*;

use super::components::{Acceleration, Velocity};
use super::constants::*;
use super::units::archer::Archer;
use super::units::components::{Corpse, Flying, Hitbox, RoughTerrain, RoughTerrainModifier, Team};
use super::units::constants::{SMELLY_SEPARATION_DISTANCE, SMELLY_SEPARATION_MULTIPLIER};
use super::units::wizard::spells::wall_of_stone::components::WallOfStone;

type SeparationQueryData = (
    Entity,
    &'static mut Transform,
    &'static Velocity,
    &'static mut super::units::components::FlockingVelocity,
    &'static Hitbox,
    &'static Team,
    Option<&'static super::units::components::FlockingModifier>,
    Has<super::units::boss::components::Boss>,
    Has<super::units::components::SmellyModifier>,
    Has<super::units::assassin::Assassin>,
);

/// Applies flocking behavior (separation, alignment, cohesion) to units.
///
/// First enforces hard collision constraint (no overlap allowed), then calculates flocking velocity.
/// Separation - Units steer away from neighbors that are too close
/// Alignment - Units steer to match the velocity of nearby neighbors
/// Cohesion - Units steer toward the average position of nearby neighbors
///
/// Defenders have alignment/cohesion disabled when not activated (returning to rally).
pub fn apply_separation(
    defenders_activated: Res<super::units::infantry::components::DefendersActivated>,
    mut units: Query<SeparationQueryData, Without<Corpse>>,
) {
    // Separation parameters are defined in constants.rs

    // Maximum Y difference for two units to interact via collision/flocking.
    // Units at different altitudes (e.g. flying vs ground) ignore each other.
    const MAX_Y_INTERACTION: f32 = 50.0;

    // Collect all unit data for comparison
    let unit_data: Vec<_> = units
        .iter()
        .map(
            |(entity, transform, velocity, _, hitbox, team, _, _, has_smelly, is_assassin)| {
                (
                    entity,
                    transform.translation,
                    Vec3::new(velocity.x, 0.0, velocity.z),
                    *hitbox,
                    *team,
                    has_smelly,
                    is_assassin,
                )
            },
        )
        .collect();

    // Pre-check: are there any smelly units on the field?
    let any_smelly = unit_data.iter().any(|(_, _, _, _, _, smelly, _)| *smelly);

    // First pass: enforce hard collision constraint (no overlap allowed)
    // Use multiple iterations to resolve stacked collisions
    for _iteration in 0..COLLISION_ITERATIONS {
        let current_positions: Vec<_> = units
            .iter()
            .map(
                |(entity, transform, _, _, hitbox, _, _, _, _, is_assassin)| {
                    (entity, transform.translation, *hitbox, is_assassin)
                },
            )
            .collect();

        for (entity, mut transform, _, _, hitbox, _, _, is_boss, _, is_assassin) in units.iter_mut()
        {
            // Boss is immovable — other units get pushed away from it, not the other way around
            if is_boss {
                continue;
            }

            let mut total_correction = Vec3::ZERO;
            let mut overlap_count = 0;

            for (other_entity, other_pos, other_hitbox, other_is_assassin) in &current_positions {
                if entity == *other_entity {
                    continue;
                }

                // Assassins pass through non-assassin units (only collide with other assassins)
                if is_assassin != *other_is_assassin {
                    continue;
                }

                // Units at different altitudes don't collide (flying vs ground)
                if (transform.translation.y - other_pos.y).abs() > MAX_Y_INTERACTION {
                    continue;
                }

                // Calculate difference on XZ plane only (ignore Y)
                let diff = Vec3::new(
                    transform.translation.x - other_pos.x,
                    0.0,
                    transform.translation.z - other_pos.z,
                );
                let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

                // Calculate minimum allowed distance (90% of combined radii = 10% max overlap)
                let min_distance =
                    (hitbox.radius + other_hitbox.radius) * (1.0 - MAX_OVERLAP_PERCENT);

                if distance < min_distance && distance > MIN_DISTANCE_THRESHOLD {
                    // Calculate how much to push apart (XZ plane only)
                    let overlap = min_distance - distance;
                    let push_direction = diff / distance;
                    // Push the full overlap distance (don't split it 50/50)
                    total_correction += push_direction * overlap;
                    overlap_count += 1;
                }
            }

            if overlap_count > 0 {
                let correction = total_correction / overlap_count as f32;
                // Apply correction only on XZ plane (preserve Y position)
                transform.translation.x += correction.x;
                transform.translation.z += correction.z;
            }
        }
    }

    // Second pass: calculate flocking velocity (separation, alignment, cohesion)
    for (
        entity,
        transform,
        _velocity,
        mut flocking_velocity,
        hitbox,
        team,
        flock_mod,
        _,
        _,
        is_assassin,
    ) in units.iter_mut()
    {
        // Defenders have alignment/cohesion disabled when not activated
        let is_defender = *team == Team::Defenders;
        let disable_flocking = is_defender && !defenders_activated.active;
        let mut separation = Vec3::ZERO;
        let mut smelly_separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut separation_count = 0;
        let mut smelly_separation_count = 0;
        let mut neighbor_count = 0;

        // Calculate forces from all neighbors
        for (
            other_entity,
            other_pos,
            other_velocity,
            other_hitbox,
            other_team,
            other_smelly,
            other_is_assassin,
        ) in &unit_data
        {
            if entity == *other_entity {
                continue;
            }

            // Assassins only flock with other assassins
            if is_assassin != *other_is_assassin {
                continue;
            }

            // Units at different altitudes don't interact (flying vs ground)
            if (transform.translation.y - other_pos.y).abs() > MAX_Y_INTERACTION {
                continue;
            }

            // Calculate difference on XZ plane only (ignore Y difference)
            let diff = Vec3::new(
                transform.translation.x - other_pos.x,
                0.0,
                transform.translation.z - other_pos.z,
            );
            let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

            // Smelly separation: same-team units flee from smelly allies at larger range
            // Tracked separately so it doesn't get diluted by normal separation averaging
            if any_smelly
                && *other_smelly
                && !team.is_enemy(other_team)
                && distance < SMELLY_SEPARATION_DISTANCE
                && distance > MIN_DISTANCE_THRESHOLD
            {
                let normalized_diff = diff / distance;
                let force = normalized_diff / distance;
                smelly_separation += force;
                smelly_separation_count += 1;
            }

            // Check if within neighbor distance
            if distance < NEIGHBOR_DISTANCE && distance > MIN_DISTANCE_THRESHOLD {
                // Separation: steer away from close neighbors
                let separation_dist = (hitbox.radius + other_hitbox.radius) + SEPARATION_DISTANCE;
                if distance < separation_dist {
                    let normalized_diff = diff / distance;
                    let force = normalized_diff / distance;
                    separation += force;
                    separation_count += 1;
                }

                // Alignment: match velocity of neighbors (already 2D)
                alignment += *other_velocity;

                // Cohesion: steer toward average position (XZ only)
                cohesion += Vec3::new(other_pos.x, 0.0, other_pos.z);

                neighbor_count += 1;
            }
        }

        // Combine and normalize flocking directions
        let mut combined_direction = Vec3::ZERO;

        let sep_mult = flock_mod.map_or(1.0, |m| m.separation);
        // Disable alignment and cohesion for defenders when not activated
        let align_mult = if disable_flocking {
            0.0
        } else {
            flock_mod.map_or(1.0, |m| m.alignment)
        };
        let coh_mult = if disable_flocking {
            0.0
        } else {
            flock_mod.map_or(1.0, |m| m.cohesion)
        };

        if separation_count > 0 {
            separation /= separation_count as f32;
            combined_direction += separation.normalize_or_zero() * SEPARATION_STRENGTH * sep_mult;
        }

        // Smelly repulsion: stored as a raw force on FlockingVelocity so it can be
        // applied directly as acceleration, bypassing the weighted direction normalization.
        if any_smelly {
            if smelly_separation_count > 0 {
                smelly_separation /= smelly_separation_count as f32;
                flocking_velocity.smelly_repulsion =
                    smelly_separation.normalize_or_zero() * SMELLY_SEPARATION_MULTIPLIER;
            } else {
                flocking_velocity.smelly_repulsion = Vec3::ZERO;
            }
        }

        if neighbor_count > 0 {
            // Alignment direction
            alignment /= neighbor_count as f32;
            combined_direction += alignment.normalize_or_zero() * ALIGNMENT_STRENGTH * align_mult;

            // Cohesion direction (XZ plane only)
            cohesion /= neighbor_count as f32;
            let cohesion_direction = Vec3::new(
                cohesion.x - transform.translation.x,
                0.0,
                cohesion.z - transform.translation.z,
            );

            // Diminish cohesion based on distance to group center
            // Closer to center = less cohesion pull
            let distance_to_center = cohesion_direction.length();
            let cohesion_factor = (distance_to_center / NEIGHBOR_DISTANCE).min(1.0);

            combined_direction += cohesion_direction.normalize_or_zero()
                * COHESION_STRENGTH
                * cohesion_factor
                * coh_mult;
        }

        // Set flocking velocity as normalized combined direction
        flocking_velocity.velocity = combined_direction.normalize_or_zero();
    }
}

/// Applies movement slowdown to units standing on rough terrain (corpses).
///
/// Units walking over corpses have their movement speed temporarily reduced.
/// This creates a tactical element where corpses affect battlefield movement.
pub fn apply_rough_terrain_slowdown(
    mut commands: Commands,
    units: Query<
        (Entity, &Transform, &Hitbox, Option<&RoughTerrainModifier>),
        (
            Without<Corpse>,
            Without<super::units::wizard::components::Wizard>,
        ),
    >,
    corpses: Query<(&Transform, &Hitbox, &RoughTerrain), With<Corpse>>,
) {
    for (entity, unit_transform, unit_hitbox, _speed_modifier) in &units {
        let mut max_slowdown: f32 = 1.0; // No slowdown by default

        // Check all corpses for overlap
        for (corpse_transform, corpse_hitbox, rough_terrain) in &corpses {
            let distance = unit_transform
                .translation
                .distance(corpse_transform.translation);
            let overlap_threshold = unit_hitbox.radius + corpse_hitbox.radius;

            if distance < overlap_threshold {
                // Apply slowdown from this corpse
                max_slowdown = max_slowdown.min(rough_terrain.slowdown_factor);
            }
        }

        // Apply the worst slowdown encountered as a RoughTerrainModifier component
        // slowdown_factor of 0.4 means 60% slower = -0.6 (negative 60%)
        if max_slowdown < 1.0 {
            let slowdown_percentage = max_slowdown - 1.0; // e.g., 0.4 - 1.0 = -0.6
            commands
                .entity(entity)
                .insert(RoughTerrainModifier(slowdown_percentage));
        } else {
            // Not on rough terrain - remove slowdown component if it exists
            commands.entity(entity).remove::<RoughTerrainModifier>();
        }
    }
}

/// Zeroes out targeting velocity when a wall blocks the line to the target.
///
/// Without this, units behind a wall push forward (via flocking/separation)
/// because their targeting still points straight through the wall.  By
/// suppressing targeting when blocked, those units follow the flow field
/// around the wall instead.
pub fn suppress_targeting_through_walls(
    walls: Query<&WallOfStone>,
    mut units: Query<
        (&Transform, &mut super::units::components::TargetingVelocity),
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

/// Pushes units out of any active Wall of Stone or Boulder entities.
///
/// Runs after movement systems to ensure units cannot walk through walls or rocks.
/// Applies a circular push-out correction to a unit's transform.
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
    walls: Query<&super::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    rocks: Query<&super::terrain::boulder::components::Boulder>,
    trees: Query<&super::terrain::tree::components::Tree>,
    mut units: Query<
        (
            &mut Transform,
            &Hitbox,
            Option<&mut super::components::Velocity>,
            Option<&super::units::components::TargetingVelocity>,
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
