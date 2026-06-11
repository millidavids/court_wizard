use bevy::prelude::*;

use crate::game::components::Velocity;
use crate::game::constants::*;
use crate::game::units::components::{Corpse, Hitbox, Team};
use crate::game::units::constants::{SMELLY_SEPARATION_DISTANCE, SMELLY_SEPARATION_MULTIPLIER};

type SeparationQueryData = (
    Entity,
    &'static mut Transform,
    &'static Velocity,
    &'static mut crate::game::units::components::FlockingVelocity,
    &'static Hitbox,
    &'static Team,
    Option<&'static crate::game::units::components::FlockingModifier>,
    Has<crate::game::units::boss::components::Boss>,
    Has<crate::game::units::components::SmellyModifier>,
    Has<crate::game::units::assassin::Assassin>,
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
    defenders_activated: Res<crate::game::units::infantry::components::DefendersActivated>,
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
