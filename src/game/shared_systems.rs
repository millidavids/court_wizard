use bevy::prelude::*;

use super::components::{Acceleration, Velocity};
use super::plugin::GlobalAttackCycle;
use super::units::components::{Health, Hitbox, MovementSpeed, Team};

/// Advances the global attack cycle timer each game frame.
///
/// This timer cycles from 0.0 to cycle_duration seconds, creating a rotating
/// schedule for unit attacks that is consistent across different frame rates.
pub fn tick_attack_cycle(time: Res<Time>, mut attack_cycle: ResMut<GlobalAttackCycle>) {
    attack_cycle.tick(time.delta_secs());
}

/// Applies flocking behavior and enforces zero hitbox overlap.
///
/// First enforces hard collision constraint (no overlap allowed), then applies flocking forces.
/// Separation - Units steer away from neighbors that are too close
/// Alignment - Units steer to match the velocity of nearby neighbors
/// Cohesion - Units steer toward the average position of nearby neighbors
pub fn apply_separation(
    mut units: Query<(
        Entity,
        &mut Transform,
        &Velocity,
        &mut Acceleration,
        &Hitbox,
    )>,
) {
    // Flocking parameters
    const NEIGHBOR_DISTANCE: f32 = 100.0;
    const SEPARATION_DISTANCE: f32 = 35.0;
    const SEPARATION_STRENGTH: f32 = 50.0;
    const ALIGNMENT_STRENGTH: f32 = 1.0;
    const COHESION_STRENGTH: f32 = 0.0;
    const MAX_OVERLAP_PERCENT: f32 = 0.0; // No overlap allowed

    // Collect all unit data for comparison
    let unit_data: Vec<_> = units
        .iter()
        .map(|(entity, transform, velocity, _, hitbox)| {
            (
                entity,
                transform.translation,
                Vec3::new(velocity.x, 0.0, velocity.z),
                *hitbox,
            )
        })
        .collect();

    // First pass: enforce hard collision constraint (no overlap allowed)
    // Use multiple iterations to resolve stacked collisions
    for _iteration in 0..4 {
        let current_positions: Vec<_> = units
            .iter()
            .map(|(entity, transform, _, _, hitbox)| (entity, transform.translation, *hitbox))
            .collect();

        for (entity, mut transform, _, _, hitbox) in units.iter_mut() {
            let mut total_correction = Vec3::ZERO;
            let mut overlap_count = 0;

            for (other_entity, other_pos, other_hitbox) in &current_positions {
                if entity == *other_entity {
                    continue;
                }

                let diff = transform.translation - *other_pos;
                let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

                // Calculate minimum allowed distance (90% of combined radii = 10% max overlap)
                let min_distance =
                    (hitbox.radius + other_hitbox.radius) * (1.0 - MAX_OVERLAP_PERCENT);

                if distance < min_distance && distance > 0.01 {
                    // Calculate how much to push apart
                    let overlap = min_distance - distance;
                    let push_direction = diff / distance;
                    // Push the full overlap distance (don't split it 50/50)
                    total_correction += push_direction * overlap;
                    overlap_count += 1;
                }
            }

            if overlap_count > 0 {
                transform.translation += total_correction / overlap_count as f32;
            }
        }
    }

    // Second pass: apply flocking forces
    for (entity, transform, _velocity, mut acceleration, hitbox) in units.iter_mut() {
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut separation_count = 0;
        let mut neighbor_count = 0;

        // Calculate forces from all neighbors
        for (other_entity, other_pos, other_velocity, other_hitbox) in &unit_data {
            if entity == *other_entity {
                continue;
            }

            let diff = transform.translation - *other_pos;
            let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

            // Check if within neighbor distance
            if distance < NEIGHBOR_DISTANCE && distance > 0.01 {
                // Separation: steer away from close neighbors
                let separation_dist = (hitbox.radius + other_hitbox.radius) + SEPARATION_DISTANCE;
                if distance < separation_dist {
                    let normalized_diff = diff / distance;
                    let force = normalized_diff / distance;
                    separation += force;
                    separation_count += 1;
                }

                // Alignment: match velocity of neighbors
                alignment += *other_velocity;

                // Cohesion: steer toward average position
                cohesion += *other_pos;

                neighbor_count += 1;
            }
        }

        // Calculate and apply final forces
        if separation_count > 0 {
            separation /= separation_count as f32;
            acceleration.add_force(separation * SEPARATION_STRENGTH);
        }

        if neighbor_count > 0 {
            // Alignment force
            alignment /= neighbor_count as f32;
            acceleration.add_force(alignment * ALIGNMENT_STRENGTH);

            // Cohesion force
            cohesion /= neighbor_count as f32;
            let cohesion_direction = cohesion - transform.translation;
            acceleration.add_force(cohesion_direction * COHESION_STRENGTH);
        }
    }
}

/// Updates units following the boids algorithm pattern.
///
/// Acceleration changes velocity, velocity changes position.
/// Acceleration is reset each frame after being applied.
/// Includes damping to reduce momentum and make units more responsive.
/// Units slow down when near enemies (100% speed at 1.2x hitbox distance, 10% when touching).
pub fn move_units(
    time: Res<Time>,
    mut all_units: Query<(
        Entity,
        &mut Transform,
        &mut Velocity,
        &mut Acceleration,
        &MovementSpeed,
        &Hitbox,
        &Team,
    )>,
) {
    const DAMPING: f32 = 0.85; // Reduces velocity each frame to prevent excessive momentum
    const MIN_SPEED_MULTIPLIER: f32 = 0.1; // Minimum speed when touching enemy (10%)
    const MAX_SPEED_MULTIPLIER: f32 = 1.0; // Maximum speed when far from enemies (100%)
    const SLOWDOWN_DISTANCE_MULTIPLIER: f32 = 1.2; // Start slowing at 1.2x combined hitbox radius

    // Collect snapshot of all unit positions BEFORE moving any units
    // This ensures symmetric movement - all units use the same frame's positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, _, _, _, hitbox, team)| {
            (entity, transform.translation, *hitbox, *team)
        })
        .collect();

    // Process all units
    for (entity, mut transform, mut velocity, mut acceleration, movement_speed, hitbox, team) in
        &mut all_units
    {
        // Apply acceleration to velocity
        velocity.x += acceleration.x * time.delta_secs();
        velocity.y += acceleration.y * time.delta_secs();
        velocity.z += acceleration.z * time.delta_secs();

        // Apply damping to reduce momentum
        velocity.x *= DAMPING;
        velocity.y *= DAMPING;
        velocity.z *= DAMPING;

        // Calculate speed multiplier based on proximity to nearest enemy
        let mut speed_multiplier = MAX_SPEED_MULTIPLIER;

        if let Some((_, nearest_pos, nearest_hitbox, _)) = unit_snapshot
            .iter()
            .filter(|(other_entity, _, _, other_team)| {
                // Skip self and only consider enemies (units on different teams)
                *other_entity != entity && *other_team != *team
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
        {
            let diff_x = transform.translation.x - nearest_pos.x;
            let diff_z = transform.translation.z - nearest_pos.z;
            let distance = (diff_x * diff_x + diff_z * diff_z).sqrt();
            let combined_radius = hitbox.radius + nearest_hitbox.radius;
            let slowdown_distance = combined_radius * SLOWDOWN_DISTANCE_MULTIPLIER;

            if distance < slowdown_distance {
                // Linearly interpolate from MAX to MIN as distance goes from slowdown_distance to combined_radius
                let t = ((distance - combined_radius) / (slowdown_distance - combined_radius))
                    .clamp(0.0, 1.0);
                speed_multiplier =
                    MIN_SPEED_MULTIPLIER + (MAX_SPEED_MULTIPLIER - MIN_SPEED_MULTIPLIER) * t;
            }
        }

        // Limit velocity to max speed with proximity multiplier
        let max_speed = movement_speed.speed * speed_multiplier;
        let horizontal_velocity = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        if horizontal_velocity > max_speed {
            let scale = max_speed / horizontal_velocity;
            velocity.x *= scale;
            velocity.z *= scale;
        }

        // Apply velocity to position
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
        transform.translation.z += velocity.z * time.delta_secs();

        // Reset acceleration for next frame
        acceleration.reset();
    }
}

/// Despawns units when their health reaches zero.
///
/// This system checks all units with Health components and removes them from the game
/// when their current health is zero or below.
pub fn despawn_dead_units(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in &query {
        if health.is_dead() {
            commands.entity(entity).despawn();
        }
    }
}

/// Cleans up all game entities when exiting the InGame state.
pub fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<super::components::OnGameplayScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
