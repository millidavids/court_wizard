use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::units::components::{
    AttackTiming, Corpse, DamageMultiplier, Effectiveness, FlockingVelocity, Health, Hitbox,
    KingAuraSpeedModifier, MovementSpeed, RoughTerrainModifier, TargetingVelocity, Team,
    Teleportable,
};

/// Spawns the King unit at the exact center of all defender spawn points.
///
/// Defender spawn points form a 2x2 grid:
/// (-1700, 1200), (-1400, 1200), (-1700, 1500), (-1400, 1500)
/// King spawns at centroid moved 100 units diagonally back from attackers
pub fn spawn_king(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut king_spawned: ResMut<KingSpawned>,
) {
    // Calculate centroid of all 4 defender spawn points
    let centroid_x = (-1700.0 + -1400.0 + -1700.0 + -1400.0) / 4.0; // = -1550
    let centroid_z = (1200.0 + 1200.0 + 1500.0 + 1500.0) / 4.0; // = 1350

    // Move King 100 units back away from attackers (diagonal: -X and +Z direction)
    // Attackers come from positive X, so moving back is negative X and positive Z
    let offset = 100.0 / 2.0f32.sqrt(); // 70.71 in each direction for 100 unit diagonal
    let spawn_x = centroid_x - offset; // More negative X (away from attackers)
    let spawn_z = centroid_z + offset; // More positive Z (towards castle)

    // Define King hitbox (larger than standard units)
    let hitbox = Hitbox::new(KING_RADIUS, KING_HITBOX_HEIGHT);

    // Spawn King as a circle billboard sized to match the hitbox
    let circle = Circle::new(hitbox.radius);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    let spawn_y = hitbox.height / 2.0 + 1.0;

    // Spawn the King unit
    let king_entity = commands
        .spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: KING_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(KING_HEALTH),
            MovementSpeed(KING_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            DamageMultiplier(KING_DAMAGE_PERCENTAGE),
            Team::Defenders,
            King,
        ))
        .insert((
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ))
        .id();

    // Spawn visual aura sphere as a child entity centered on the King
    // The sphere's radius exactly represents the 3D distance check used by the aura system
    let aura_sphere = Sphere::new(KING_AURA_RADIUS);
    commands
        .spawn((
            Mesh3d(meshes.add(aura_sphere)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.6, 0.0, 0.05), // Very transparent orange sphere
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                cull_mode: None, // Visible from both sides
                ..default()
            })),
            // Center sphere on King (relative position 0,0,0 since it's a child entity)
            // This accurately represents the 3D spherical distance check
            Transform::from_xyz(0.0, 0.0, 0.0),
            OnGameplayScreen,
        ))
        .set_parent_in_place(king_entity);

    // Mark that King has been spawned
    king_spawned.0 = true;
}

/// Updates King targeting velocity toward nearest enemy.
///
/// The King always moves directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
pub fn update_king_targeting(
    mut commands: Commands,
    mut king: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<King>, Without<Corpse>),
    >,
    all_units: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    // Collect snapshot of all unit positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update King's targeting velocity
    for (entity, transform, team, mut targeting_velocity) in &mut king {
        // Find nearest enemy
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity
                    && match (*team, other_team) {
                        (Team::Undead, Team::Undead) => false,
                        (Team::Undead, _) => true,
                        (_, Team::Undead) => true,
                        _ => *other_team != *team,
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

        // Set targeting velocity toward target (normalized direction)
        if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
            let direction = (target_pos - transform.translation).normalize_or_zero();
            targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);

            // Store distance for formation weighting
            let distance = transform.translation.distance(target_pos);
            targeting_velocity.distance_to_target = distance;

            // Check if enemy is in melee range
            if distance < MELEE_SLOWDOWN_DISTANCE {
                commands
                    .entity(entity)
                    .insert(crate::game::units::components::InMelee(enemy_team));
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        }
    }
}

/// King-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// King slows down when in melee to prevent erratic movement.
pub fn king_movement(
    time: Res<Time>,
    mut king_units: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&KingAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
        ),
        With<King>,
    >,
) {
    let delta = time.delta_secs();

    // Process King unit
    for (
        mut transform,
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
    ) in &mut king_units
    {
        // Weight targeting vs flocking based on distance to target
        // When far from target: prioritize flocking (stay in formation)
        // When close to target: prioritize targeting (engage enemy)
        // Transition happens around 500 units distance
        let targeting_weight =
            (1.0 - (targeting_velocity.distance_to_target / 500.0).min(1.0)).max(0.2); // Minimum 20% targeting weight
        let flocking_weight = 1.0 - targeting_weight;

        // Combine targeting and flocking velocities with distance-based weighting
        let weighted_direction = (targeting_velocity.velocity * targeting_weight
            + flocking_velocity.velocity * flocking_weight)
            .normalize_or_zero();

        // Calculate speed modifiers early to apply to acceleration
        let aura_percentage = aura_modifier.map_or(0.0, |m| m.0);
        let terrain_percentage = terrain_modifier.map_or(0.0, |m| m.0);
        let total_percentage = aura_percentage + terrain_percentage;
        let speed_multiplier = 1.0 + total_percentage;

        // Apply as acceleration force with speed modifiers
        acceleration.add_force(weighted_direction * STEERING_FORCE * speed_multiplier);

        // Apply acceleration to velocity
        velocity.x += acceleration.x * delta;
        velocity.z += acceleration.z * delta;

        // Apply damping to smooth movement
        velocity.x *= VELOCITY_DAMPING;
        velocity.z *= VELOCITY_DAMPING;

        // Calculate max speed with effectiveness, modifiers (aura + terrain), and melee slowdown
        let mut max_speed = movement_speed.0 * effectiveness.multiplier() * speed_multiplier;
        if in_melee.is_some() {
            max_speed *= MELEE_SLOWDOWN_FACTOR;
        }

        // King's absolute speed cap - 90% of standard unit movement speed
        max_speed = max_speed.min(UNIT_MOVEMENT_SPEED * 0.9);

        // Cap velocity to maximum speed
        let velocity_vec = Vec3::new(velocity.x, 0.0, velocity.z);
        let current_speed = velocity_vec.length();
        if current_speed > max_speed {
            let normalized = velocity_vec.normalize();
            velocity.x = normalized.x * max_speed;
            velocity.z = normalized.z * max_speed;
        }

        // Apply velocity to position (only XZ plane - Y stays fixed at spawn height)
        transform.translation.x += velocity.x * delta;
        transform.translation.z += velocity.z * delta;

        // Reset acceleration for next frame
        acceleration.reset();
    }
}

/// King cohesion aura system.
///
/// Applies a dynamic cohesion force to all nearby units, pulling them toward the King.
/// The force strength increases when enemies are near (threatened) and decreases when safe.
/// Defenders are drawn to protect the King, attackers are drawn to kill the King.
/// Also applies/removes damage and speed buffs to defenders within aura range.
/// The King himself also receives the aura buffs.
pub fn king_cohesion_aura(
    mut commands: Commands,
    king_query: Query<(Entity, &Transform), (With<King>, Without<Corpse>)>,
    mut all_affected_units: Query<
        (Entity, &Transform, &Team, &mut FlockingVelocity),
        (Without<King>, Without<Corpse>),
    >,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // Get King entity and position (should only be one)
    let Ok((king_entity, king_transform)) = king_query.single() else {
        return;
    };

    let king_pos = king_transform.translation;

    // Find nearest enemy to King
    let nearest_enemy_distance = all_units
        .iter()
        .filter(|(_, team)| **team != Team::Defenders)
        .map(|(transform, _)| transform.translation.distance(king_pos))
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(f32::MAX);

    // Calculate threat level: interpolate between BASE and THREATENED
    // If enemy is far (> AURA_RADIUS), use BASE
    // If enemy is close (< AURA_RADIUS), interpolate to THREATENED
    let threat_factor = if nearest_enemy_distance > KING_AURA_RADIUS {
        0.0
    } else {
        1.0 - (nearest_enemy_distance / KING_AURA_RADIUS)
    };

    let cohesion_strength =
        KING_COHESION_BASE + (KING_COHESION_THREATENED - KING_COHESION_BASE) * threat_factor;

    // Apply cohesion force to all units within aura radius, damage and speed buffs only to defenders
    for (entity, unit_transform, team, mut flocking_velocity) in &mut all_affected_units {
        let unit_pos = unit_transform.translation;
        let distance_to_king = unit_pos.distance(king_pos);

        // Check if unit is within aura radius
        if distance_to_king < KING_AURA_RADIUS && distance_to_king > 0.1 {
            // Apply cohesion force only to defenders (they protect the King)
            // Attackers use their normal targeting behavior to attack the King
            if *team == Team::Defenders {
                // Calculate direction toward King
                let to_king = (king_pos - unit_pos).normalize_or_zero();

                // Add cohesion force to flocking velocity
                // Scale by distance (stronger pull when closer to edge of aura)
                let distance_factor = distance_to_king / KING_AURA_RADIUS;
                let cohesion_force = to_king * cohesion_strength * distance_factor;

                flocking_velocity.velocity += Vec3::new(cohesion_force.x, 0.0, cohesion_force.z);

                // Re-normalize to maintain consistent influence
                flocking_velocity.velocity = flocking_velocity.velocity.normalize_or_zero();

                // Apply damage and speed buffs to defenders (just set to fixed value)
                commands
                    .entity(entity)
                    .insert(DamageMultiplier(KING_AURA_DAMAGE_PERCENTAGE));
                commands
                    .entity(entity)
                    .insert(KingAuraSpeedModifier(KING_AURA_SPEED_PERCENTAGE));
            }
        } else if *team == Team::Defenders {
            // Remove aura buffs if defender is outside aura
            commands.entity(entity).remove::<DamageMultiplier>();
            commands.entity(entity).remove::<KingAuraSpeedModifier>();
        }
    }

    // Apply aura buffs to the King himself (he's always in his own aura)
    // The King gets speed buff but not damage buff (he already has base damage multiplier)
    commands
        .entity(king_entity)
        .insert(KingAuraSpeedModifier(KING_AURA_SPEED_PERCENTAGE));
}
