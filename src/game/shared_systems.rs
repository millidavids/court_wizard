use bevy::prelude::*;

use crate::config::GameConfig;

use super::components::{Acceleration, Velocity};
use super::constants::*;
use super::plugin::GlobalAttackCycle;
use super::resources::CurrentLevel;
use super::units::components::{
    AttackTiming, Corpse, DamageMultiplier, Effectiveness, Health, Hitbox, MovementSpeed,
    RoughTerrain, RoughTerrainModifier, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use super::units::king::components::KingSpawned;

/// Advances the global attack cycle timer each game frame.
///
/// This timer cycles from 0.0 to cycle_duration seconds, creating a rotating
/// schedule for unit attacks that is consistent across different frame rates.
pub fn tick_attack_cycle(time: Res<Time>, mut attack_cycle: ResMut<GlobalAttackCycle>) {
    attack_cycle.tick(time.delta_secs());
}

/// Initializes the current level from saved config.
///
/// This system runs on OnEnter(AppState::InGame) to restore the player's
/// current level from their last session.
pub fn init_level_from_config(mut current_level: ResMut<CurrentLevel>, config: Res<GameConfig>) {
    current_level.0 = config.current_level;
}

/// Calculates effectiveness for all units based on melee proximity.
///
/// Effectiveness is modified by:
/// - Number of allies in melee range (positive effect: +10% per ally)
/// - Number of enemies in melee range (negative effect: -15% per enemy)
///
/// The effectiveness coefficient is applied to both movement speed and attack damage
/// in their respective systems. This encourages tactical positioning and rewards
/// units that fight together while penalizing isolated units.
pub fn calculate_effectiveness(
    mut units: Query<(Entity, &Transform, &Hitbox, &Team, &mut Effectiveness), Without<Corpse>>,
) {
    // Collect snapshot for symmetric calculations
    let unit_data: Vec<_> = units
        .iter()
        .map(|(entity, transform, hitbox, team, _)| (entity, transform.translation, *hitbox, *team))
        .collect();

    // Calculate effectiveness for each unit
    for (entity, transform, hitbox, team, mut effectiveness) in units.iter_mut() {
        let mut ally_count = 0;
        let mut enemy_count = 0;

        for (other_entity, other_pos, other_hitbox, other_team) in &unit_data {
            if *other_entity == entity {
                continue;
            }

            // Calculate XZ plane distance
            let dx = transform.translation.x - other_pos.x;
            let dz = transform.translation.z - other_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            // Use same melee range formula as combat
            let melee_range = (hitbox.radius + other_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if distance <= melee_range {
                // Team logic matches combat system
                let is_enemy = match (*team, *other_team) {
                    (Team::Undead, Team::Undead) => false,
                    (Team::Undead, _) => true,
                    (_, Team::Undead) => true,
                    _ => other_team != team,
                };

                if is_enemy {
                    enemy_count += 1;
                } else {
                    ally_count += 1;
                }
            }
        }

        effectiveness.recalculate(ally_count, enemy_count);
    }
}

/// Applies flocking behavior and enforces zero hitbox overlap.
///
/// First enforces hard collision constraint (no overlap allowed), then calculates flocking velocity.
/// Separation - Units steer away from neighbors that are too close
/// Alignment - Units steer to match the velocity of nearby neighbors
/// Cohesion - Units steer toward the average position of nearby neighbors
pub fn apply_separation(
    mut units: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
            &mut super::units::components::FlockingVelocity,
            &Hitbox,
            Option<&super::units::components::FlockingModifier>,
        ),
        Without<Corpse>,
    >,
) {
    // Flocking parameters are defined in constants.rs

    // Collect all unit data for comparison
    let unit_data: Vec<_> = units
        .iter()
        .map(|(entity, transform, velocity, _, hitbox, _)| {
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
            .map(|(entity, transform, _, _, hitbox, _)| (entity, transform.translation, *hitbox))
            .collect();

        for (entity, mut transform, _, _, hitbox, _) in units.iter_mut() {
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

    // Second pass: calculate flocking velocity
    for (entity, transform, _velocity, mut flocking_velocity, hitbox, flock_mod) in units.iter_mut()
    {
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

        // Combine and normalize flocking directions
        let mut combined_direction = Vec3::ZERO;

        let sep_mult = flock_mod.map_or(1.0, |m| m.separation);
        let align_mult = flock_mod.map_or(1.0, |m| m.alignment);
        let coh_mult = flock_mod.map_or(1.0, |m| m.cohesion);

        if separation_count > 0 {
            separation /= separation_count as f32;
            combined_direction += separation.normalize_or_zero() * SEPARATION_STRENGTH * sep_mult;
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

pub fn combat(
    attack_cycle: Res<GlobalAttackCycle>,
    mut all_units: Query<(
        Entity,
        &Transform,
        &Hitbox,
        &Team,
        &mut AttackTiming,
        &Effectiveness,
        Option<&DamageMultiplier>,
    )>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - APPROX_FRAME_TIME).max(0.0);

    // Collect snapshot of all units for enemy detection
    let units_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, hitbox, team, _, _, _)| {
            (entity, transform.translation, *hitbox, *team)
        })
        .collect();

    // Process each unit's combat
    for (
        attacker_entity,
        attacker_transform,
        attacker_hitbox,
        attacker_team,
        mut attack_timing,
        effectiveness,
        damage_mult,
    ) in &mut all_units
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
                // Apply effectiveness and damage percentage
                // DamageMultiplier stores percentage bonus (0.5 = +50%, 1.0 = +100%)
                // Convert to multiplier: damage * (1.0 + percentage)
                let damage_percentage = damage_mult.map_or(0.0, |d| d.0);
                let damage_multiplier = 1.0 + damage_percentage;
                let modified_damage =
                    ATTACK_DAMAGE * effectiveness.multiplier() * damage_multiplier;
                apply_damage_to_unit(&mut target_health, temp_hp.as_deref_mut(), modified_damage);
                attack_timing.record_attack(current_time);
            }
        }
    }
}

/// Converts dead units to corpses instead of despawning them.
///
/// When a unit's health reaches zero, this system grays out the sprite based on team
/// and converts the unit into a corpse that slows living units walking over it.
/// Also records the kill in the kill statistics resource.
pub fn convert_dead_to_corpses(
    mut commands: Commands,
    mut kill_stats: ResMut<super::resources::KillStats>,
    query: Query<(Entity, &Health, &Team, &Transform), Without<Corpse>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (entity, health, team, transform) in &query {
        if health.is_dead() {
            // Record the kill
            kill_stats.record_kill(*team);
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
                    slowdown_factor: 0.4,
                }); // 60% speed reduction

            // Mark undead corpses as permanent (cannot be resurrected)
            if *team == Team::Undead {
                entity_commands.insert(super::units::components::PermanentCorpse);
            }

            entity_commands
                .remove::<Velocity>() // Stop moving
                .remove::<Acceleration>() // No forces
                .remove::<MovementSpeed>() // Can't move
                .remove::<AttackTiming>() // Can't attack
                .remove::<Hitbox>() // Remove collision
                .remove::<crate::game::components::Billboard>(); // Remove billboard so corpse stays flat
        }
    }
}

/// Cleans up all game entities when exiting the InGame state.
pub fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<super::components::OnGameplayScreen>>,
) {
    // Don't reset level - it persists between sessions via config
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Cleans up game entities when replaying (transitioning from GameOver to Running).
///
/// This system runs on OnExit(InGameState::GameOver) and despawns all game entities
/// in preparation for re-spawning them fresh.
pub fn cleanup_for_replay(
    mut commands: Commands,
    gameplay_entities: Query<Entity, With<super::components::OnGameplayScreen>>,
) {
    for entity in &gameplay_entities {
        commands.entity(entity).despawn();
    }
}

/// Applies a steering force to units approaching walls so they navigate around them.
pub fn apply_wall_avoidance(
    walls: Query<&super::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    mut units: Query<(&Transform, &Velocity, &mut Acceleration, &Hitbox), Without<Corpse>>,
) {
    const AVOIDANCE_DISTANCE: f32 = 80.0; // How far ahead units look for walls
    const AVOIDANCE_FORCE: f32 = 800.0; // Strength of the avoidance steering

    for (transform, velocity, mut acceleration, hitbox) in &mut units {
        let vel = Vec3::new(velocity.x, 0.0, velocity.z);
        let speed = vel.length();
        if speed < 1.0 {
            continue;
        }
        let vel_dir = vel / speed;

        // Check if the unit's projected position will be inside a wall
        let look_ahead = transform.translation + vel_dir * AVOIDANCE_DISTANCE;

        for wall in &walls {
            // Check if look-ahead point is inside the wall
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
                // Unit is heading into the wall — steer along the wall edge
                // Choose the perpendicular direction that requires least deviation
                let steer = if forward_pen < right_pen {
                    // Closer to a forward edge — steer along the right axis
                    wall.right * right_proj.signum()
                } else {
                    // Closer to a right edge — steer along the forward axis
                    wall.forward * forward_proj.signum()
                };

                // Scale force by how close we are to the wall
                let proximity = 1.0 - (forward_pen.min(right_pen) / AVOIDANCE_DISTANCE).min(1.0);
                acceleration.add_force(steer * AVOIDANCE_FORCE * proximity);
            }
        }
    }
}

/// Pushes units out of any active Wall of Stone entities.
///
/// Runs after movement systems to ensure units cannot walk through walls.
pub fn enforce_wall_collision(
    walls: Query<&super::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    mut units: Query<(&mut Transform, &Hitbox), Without<Corpse>>,
) {
    for (mut transform, hitbox) in &mut units {
        for wall in &walls {
            if let Some(corrected) = wall.push_out(transform.translation, hitbox.radius) {
                transform.translation.x = corrected.x;
                transform.translation.z = corrected.z;
            }
        }
    }
}

/// Resets game resources when replaying (transitioning from GameOver to Running).
///
/// This system runs on OnExit(InGameState::GameOver) and resets resources like
/// the attack cycle timer and defender activation status.
pub fn reset_resources_for_replay(
    mut attack_cycle: ResMut<super::plugin::GlobalAttackCycle>,
    mut defenders_activated: ResMut<super::units::infantry::components::DefendersActivated>,
    mut king_spawned: ResMut<KingSpawned>,
) {
    attack_cycle.current_time = 0.0;
    defenders_activated.active = false;
    king_spawned.0 = false;
}
