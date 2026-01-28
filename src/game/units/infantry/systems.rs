use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::units::components::{
    AttackTiming, Effectiveness, FlockingVelocity, Health, Hitbox, MovementSpeed,
    TargetingVelocity, Team, Teleportable,
};

/// Spawns initial defenders when entering the game.
///
/// Spawns defenders in a 2×2 grid formation under the castle.
/// Infantry spawn at the 3 closest points to attackers (rightmost points).
/// Distributes units evenly across the 3 front-line spawn points.
pub fn spawn_initial_defenders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Infantry spawn at 3 front-line points (closest to attackers = rightmost)
    // Skip index 2 which is (-1750, 1550) - the back-left point
    let infantry_spawn_points = [
        DEFENDER_SPAWN_POINTS[0], // (-1750, 1150) - front-left
        DEFENDER_SPAWN_POINTS[1], // (-1350, 1150) - front-right
        DEFENDER_SPAWN_POINTS[3], // (-1350, 1550) - back-right
    ];

    let units_per_point = INITIAL_DEFENDER_COUNT / infantry_spawn_points.len() as u32;

    for &(spawn_x, spawn_z) in &infantry_spawn_points {
        for i in 0..units_per_point {
            // Define defender hitbox (cylinder) - this determines sprite size
            let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);

            // Spawn defender as a circle billboard sized to match the hitbox
            let circle = Circle::new(hitbox.radius);

            // Distribute spawns in a circular pattern around this spawn point
            let offset = i as f32 * SPAWN_OFFSET_MULTIPLIER;
            let final_x = spawn_x + (offset.sin() * SPAWN_DISTRIBUTION_RADIUS);
            let final_z = spawn_z + (offset.cos() * SPAWN_DISTRIBUTION_RADIUS);

            // Position unit so bottom edge is 1 unit above battlefield (Y=0)
            let spawn_y = hitbox.height / 2.0 + 1.0;

            commands
                .spawn((
                    Mesh3d(meshes.add(circle)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: DEFENDER_COLOR,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed::new(UNIT_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Defenders,
                    Infantry,
                ))
                .insert((
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
        }
    }
}

/// Updates infantry targeting velocity toward nearest enemy.
///
/// Infantry always move directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
pub fn update_infantry_targeting(
    mut commands: Commands,
    mut infantry: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut crate::game::units::components::TargetingVelocity,
        ),
        (
            With<Infantry>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
    all_units: Query<(Entity, &Transform, &Team), Without<crate::game::units::components::Corpse>>,
) {
    // Collect snapshot of all unit positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update each infantry's targeting velocity
    for (entity, transform, team, mut targeting_velocity) in &mut infantry {
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

/// Infantry-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// Units slow down when in melee to prevent erratic movement.
pub fn infantry_movement(
    time: Res<Time>,
    mut infantry_units: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            Option<&crate::game::units::components::InMelee>,
        ),
        With<Infantry>,
    >,
) {
    let delta = time.delta_secs();

    // Process each infantry unit
    for (
        mut transform,
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        in_melee,
    ) in &mut infantry_units
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

        // Apply as acceleration force
        acceleration.add_force(weighted_direction * STEERING_FORCE);

        // Apply acceleration to velocity
        velocity.x += acceleration.x * delta;
        velocity.z += acceleration.z * delta;

        // Apply damping to smooth movement
        velocity.x *= VELOCITY_DAMPING;
        velocity.z *= VELOCITY_DAMPING;

        // Calculate max speed with melee slowdown
        let mut max_speed = movement_speed.speed * effectiveness.multiplier();
        if in_melee.is_some() {
            max_speed *= MELEE_SLOWDOWN_FACTOR;
        }

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

/// Spawns initial attackers when entering the game.
///
/// Spawns attackers in a 2×2 grid formation in the northeast corner.
/// Infantry spawn at the 3 closest points to defenders (leftmost points).
/// Distributes units evenly across the 3 front-line spawn points.
pub fn spawn_initial_attackers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Infantry spawn at 3 front-line points (closest to defenders = leftmost)
    // Skip index 1 which is (1600, -1600) - the back-right point from camera view
    let infantry_spawn_points = [
        ATTACKER_SPAWN_POINTS[0], // (1200, -1600) - front-left
        ATTACKER_SPAWN_POINTS[2], // (1200, -1200) - back-left
        ATTACKER_SPAWN_POINTS[3], // (1600, -1200) - front-right
    ];

    let units_per_point = INITIAL_ATTACKER_COUNT / infantry_spawn_points.len() as u32;

    for &(spawn_x, spawn_z) in &infantry_spawn_points {
        for i in 0..units_per_point {
            // Define attacker hitbox (cylinder) - this determines sprite size
            let hitbox = Hitbox::new(UNIT_RADIUS, ATTACKER_HITBOX_HEIGHT);

            // Spawn attacker as a circle billboard sized to match the hitbox
            let circle = Circle::new(hitbox.radius);

            // Distribute spawns in a circular pattern around this spawn point
            let offset = i as f32 * SPAWN_OFFSET_MULTIPLIER;
            let final_x = spawn_x + (offset.sin() * SPAWN_DISTRIBUTION_RADIUS);
            let final_z = spawn_z + (offset.cos() * SPAWN_DISTRIBUTION_RADIUS);

            // Position unit so bottom edge is 1 unit above battlefield (Y=0)
            let spawn_y = hitbox.height / 2.0 + 1.0;

            commands
                .spawn((
                    Mesh3d(meshes.add(circle)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: ATTACKER_COLOR,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed::new(UNIT_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Attackers,
                    Infantry,
                ))
                .insert((
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
        }
    }
}
