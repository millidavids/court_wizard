use bevy::prelude::*;

use super::components::{Acceleration, Velocity};
use super::constants::*;
use super::plugin::GlobalAttackCycle;
use super::units::components::{
    AttackTiming, Corpse, Health, Hitbox, MovementSpeed, RoughTerrain, Team, TemporaryHitPoints,
    apply_damage_to_unit,
};

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
    mut units: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
            &mut Acceleration,
            &Hitbox,
        ),
        Without<Corpse>,
    >,
) {
    // Flocking parameters are defined in constants.rs

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
    for _iteration in 0..COLLISION_ITERATIONS {
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

            // Calculate difference on XZ plane only (ignore Y difference)
            let diff = Vec3::new(
                transform.translation.x - other_pos.x,
                0.0,
                transform.translation.z - other_pos.z,
            );
            let distance = (diff.x * diff.x + diff.z * diff.z).sqrt();

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

        // Calculate and apply final forces
        if separation_count > 0 {
            separation /= separation_count as f32;
            acceleration.add_force(separation * SEPARATION_STRENGTH);
        }

        if neighbor_count > 0 {
            // Alignment force
            alignment /= neighbor_count as f32;
            acceleration.add_force(alignment * ALIGNMENT_STRENGTH);

            // Cohesion force (XZ plane only)
            cohesion /= neighbor_count as f32;
            let cohesion_direction = Vec3::new(
                cohesion.x - transform.translation.x,
                0.0,
                cohesion.z - transform.translation.z,
            );
            acceleration.add_force(cohesion_direction * COHESION_STRENGTH);
        }
    }
}

/// Applies movement slowdown to units standing on rough terrain (corpses).
///
/// Units walking over corpses have their movement speed temporarily reduced.
/// This creates a tactical element where corpses affect battlefield movement.
pub fn apply_rough_terrain_slowdown(
    mut units: Query<
        (&Transform, &Hitbox, &mut MovementSpeed),
        (
            Without<Corpse>,
            Without<super::units::wizard::components::Wizard>,
        ),
    >,
    corpses: Query<(&Transform, &Hitbox, &RoughTerrain), With<Corpse>>,
) {
    for (unit_transform, unit_hitbox, mut movement_speed) in &mut units {
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

        // Apply the worst slowdown encountered
        if max_slowdown < 1.0 {
            movement_speed.speed *= max_slowdown;
        }
    }
}

/// Updates units following the boids algorithm pattern.
///
/// Also applies infantry steering forces toward nearest enemy and melee randomness.
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
    infantry_query: Query<Entity, With<super::units::infantry::components::Infantry>>,
    mut defenders_activated: ResMut<super::units::infantry::components::DefendersActivated>,
    corpse_query: Query<Entity, With<Corpse>>,
) {
    // Movement parameters are defined in constants.rs

    // Collect snapshot of all unit positions BEFORE moving any units
    // This ensures symmetric movement - all units use the same frame's positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, _, _, _, hitbox, team)| {
            (entity, transform.translation, *hitbox, *team)
        })
        .collect();

    // Check defender activation (infantry-specific logic)
    if !defenders_activated.active {
        'activation: for &(_, pos_a, _, team_a) in &unit_snapshot {
            if team_a != Team::Defenders {
                continue;
            }
            for &(_, pos_b, _, team_b) in &unit_snapshot {
                if team_b == Team::Attackers && pos_a.distance(pos_b) < DEFENDER_ACTIVATION_DISTANCE
                {
                    defenders_activated.active = true;
                    break 'activation;
                }
            }
        }
    }

    // Process all units
    for (entity, mut transform, mut velocity, mut acceleration, movement_speed, hitbox, team) in
        &mut all_units
    {
        // Infantry-specific: Add steering force toward nearest enemy
        if infantry_query.contains(entity) {
            // Skip inactive defenders
            if *team == Team::Defenders && !defenders_activated.active {
                // Don't add steering, but continue to process movement/damping
            } else {
                // Find nearest enemy for this infantry unit
                let nearest_enemy = unit_snapshot
                    .iter()
                    .filter(|(other_entity, _, _, other_team)| {
                        *other_entity != entity
                            && match (*team, other_team) {
                                (Team::Undead, Team::Undead) => false,
                                (Team::Undead, _) => true,
                                (_, Team::Undead) => true,
                                _ => other_team != team,
                            }
                    })
                    .min_by(|a, b| {
                        let dist_a = (transform.translation.x - a.1.x).powi(2)
                            + (transform.translation.z - a.1.z).powi(2);
                        let dist_b = (transform.translation.x - b.1.x).powi(2)
                            + (transform.translation.z - b.1.z).powi(2);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                if let Some(&(_, enemy_pos, enemy_hitbox, _)) = nearest_enemy {
                    let diff_xz = Vec3::new(
                        enemy_pos.x - transform.translation.x,
                        0.0,
                        enemy_pos.z - transform.translation.z,
                    );
                    let distance = (diff_xz.x * diff_xz.x + diff_xz.z * diff_xz.z).sqrt();
                    let melee_range =
                        (hitbox.radius + enemy_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

                    // Add random movement in melee range
                    if distance < melee_range {
                        let seed = transform.translation.x
                            * super::units::constants::MELEE_RANDOM_SEED_X_MULTIPLIER
                            + transform.translation.z
                                * super::units::constants::MELEE_RANDOM_SEED_Z_MULTIPLIER;
                        let t = time.elapsed_secs();
                        let random_angle = (t * super::units::constants::MELEE_RANDOM_FREQ_PRIMARY
                            + seed)
                            .sin()
                            * super::units::constants::MELEE_RANDOM_AMPLITUDE_PRIMARY
                            + (t * super::units::constants::MELEE_RANDOM_FREQ_SECONDARY
                                + seed
                                    * super::units::constants::MELEE_RANDOM_SEED_FREQ_MULTIPLIER)
                                .cos();
                        let random_x = random_angle.sin() * MELEE_RANDOM_FORCE * time.delta_secs();
                        let random_z = random_angle.cos() * MELEE_RANDOM_FORCE * time.delta_secs();
                        acceleration.add_force(Vec3::new(random_x, 0.0, random_z));
                    }

                    // Add steering force toward enemy
                    let steering = diff_xz.normalize_or_zero() * STEERING_FORCE;
                    acceleration.add_force(steering);
                }
            }
        }

        // Apply acceleration to velocity (only XZ plane - units don't move vertically)
        velocity.x += acceleration.x * time.delta_secs();
        velocity.z += acceleration.z * time.delta_secs();

        // Apply damping to reduce momentum
        velocity.x *= VELOCITY_DAMPING;
        velocity.z *= VELOCITY_DAMPING;

        // Calculate speed multiplier based on proximity to nearest enemy
        let mut speed_multiplier = MAX_SPEED_MULTIPLIER;

        if let Some((_, nearest_pos, nearest_hitbox, _)) = unit_snapshot
            .iter()
            .filter(|(other_entity, _, _, other_team)| {
                // Skip self, only consider enemies (units on different teams), and exclude corpses
                *other_entity != entity
                    && *other_team != *team
                    && !corpse_query.contains(*other_entity)
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

        // Apply velocity to position (only XZ plane - Y stays fixed at spawn height)
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.z += velocity.z * time.delta_secs();

        // Reset acceleration for next frame
        acceleration.reset();
    }
}

/// Unified combat system for all units.
///
/// Units attack the nearest enemy within range. Attacks are time-based using the global
/// attack cycle to naturally stagger attacks across all units.
pub fn combat(
    attack_cycle: Res<GlobalAttackCycle>,
    mut all_units: Query<(Entity, &Transform, &Hitbox, &Team, &mut AttackTiming)>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - APPROX_FRAME_TIME).max(0.0);

    // Collect snapshot of all units for enemy detection
    let units_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, hitbox, team, _)| (entity, transform.translation, *hitbox, *team))
        .collect();

    // Process each unit's combat
    for (attacker_entity, attacker_transform, attacker_hitbox, attacker_team, mut attack_timing) in
        &mut all_units
    {
        // Find nearest enemy within attack range
        if let Some((target_entity, _, _)) = units_snapshot
            .iter()
            .filter(|(entity, _, _, team)| {
                // Skip self and apply team-based targeting logic
                *entity != attacker_entity
                    && match (attacker_team, team) {
                        // Undead don't attack each other
                        (Team::Undead, Team::Undead) => false,
                        // Undead attack living
                        (Team::Undead, _) => true,
                        // Living attack undead
                        (_, Team::Undead) => true,
                        // Normal team logic
                        _ => *team != *attacker_team,
                    }
            })
            .filter_map(|(entity, target_pos, target_hitbox, _)| {
                let distance = attacker_transform.translation.distance(*target_pos);
                let attack_range =
                    (attacker_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
                if distance <= attack_range {
                    Some((entity, target_pos, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        {
            // Attack if we're in the unit's attack window
            if attack_timing.can_attack(current_time, last_time)
                && let Ok((mut target_health, mut temp_hp)) = health_query.get_mut(*target_entity)
            {
                apply_damage_to_unit(&mut target_health, temp_hp.as_deref_mut(), ATTACK_DAMAGE);
                attack_timing.record_attack(current_time);
            }
        }
    }
}

/// Converts dead units to corpses instead of despawning them.
///
/// When a unit's health reaches zero, this system grays out the sprite based on team
/// and converts the unit into a corpse that slows living units walking over it.
pub fn convert_dead_to_corpses(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Team, &Transform), Without<Corpse>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (entity, health, team, transform) in &query {
        if health.is_dead() {
            // Get existing material handle and gray out the sprite based on team
            if let Ok(material_handle) = material_query.get(entity)
                && let Some(material) = materials.get_mut(&material_handle.0)
            {
                material.base_color = match team {
                    Team::Defenders => Color::srgb(0.6, 0.6, 0.4), // Grayish yellow
                    Team::Attackers => Color::srgb(0.6, 0.4, 0.4), // Grayish red
                    Team::Undead => Color::srgb(0.4, 0.5, 0.4),    // Grayish green
                };
            }

            // Create a new transform for the corpse: lay flat on ground at Y=1
            // Rotate -90 degrees around X axis to make it face upward
            let corpse_transform =
                Transform::from_xyz(transform.translation.x, 1.0, transform.translation.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

            // Add corpse marker and rough terrain effect
            let mut entity_commands = commands.entity(entity);
            entity_commands
                .insert(Corpse)
                .insert(corpse_transform)
                .insert(RoughTerrain {
                    slowdown_factor: 0.6,
                }); // 40% speed reduction

            // Mark undead corpses as permanent (cannot be resurrected)
            if *team == Team::Undead {
                entity_commands.insert(super::units::components::PermanentCorpse);
            }

            entity_commands
                .remove::<Velocity>() // Stop moving
                .remove::<Acceleration>() // No forces
                .remove::<MovementSpeed>() // Can't move
                .remove::<AttackTiming>() // Can't attack
                .remove::<Hitbox>(); // Remove collision
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
