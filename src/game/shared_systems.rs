use bevy::prelude::*;

use crate::config::GameConfig;

use super::cauldron::components::{
    CauldronDamageBonus, CauldronDamageResistance, CauldronSpeedModifier,
};
use super::cauldron::resources::CauldronBuffs;
use super::components::{Acceleration, Velocity};
use super::constants::*;
use super::plugin::GlobalAttackCycle;
use super::resources::CurrentLevel;
use super::units::archer::Archer;
use super::units::components::{
    AttackTiming, Corpse, DamageMultiplier, Effectiveness, EliteDamageBonus, Health, Hitbox,
    MovementSpeed, ResidualFireDamaged, RoughTerrain, RoughTerrainModifier, SpellDamaged, Team,
    TemporaryHitPoints, apply_damage_to_unit,
};
use super::units::infantry::components::Infantry;
use super::units::king::components::KingSpawned;

use crate::game::achievements::messages::{
    DefenderKilledBySpellMessage, EnemyKilledMessage, ScorchedEarthMessage,
};

/// Advances the global attack cycle timer each game frame.
///
/// This timer cycles from 0.0 to cycle_duration seconds, creating a rotating
/// schedule for unit attacks that is consistent across different frame rates.
pub fn tick_attack_cycle(time: Res<Time>, mut attack_cycle: ResMut<GlobalAttackCycle>) {
    attack_cycle.tick(time.delta_secs());
}

/// Ticks the elapsed game time for achievement tracking.
pub fn tick_elapsed_time(time: Res<Time>, mut kill_stats: ResMut<super::resources::KillStats>) {
    kill_stats.elapsed_time += time.delta_secs();
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
    mut units: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
            &mut super::units::components::FlockingVelocity,
            &Hitbox,
            &Team,
            Option<&super::units::components::FlockingModifier>,
        ),
        Without<Corpse>,
    >,
) {
    // Separation parameters are defined in constants.rs

    // Collect all unit data for comparison
    let unit_data: Vec<_> = units
        .iter()
        .map(|(entity, transform, velocity, _, hitbox, _, _)| {
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
            .map(|(entity, transform, _, _, hitbox, _, _)| (entity, transform.translation, *hitbox))
            .collect();

        for (entity, mut transform, _, _, hitbox, _, _) in units.iter_mut() {
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

    // Second pass: calculate flocking velocity (separation, alignment, cohesion)
    for (entity, transform, _velocity, mut flocking_velocity, hitbox, team, flock_mod) in
        units.iter_mut()
    {
        // Defenders have alignment/cohesion disabled when not activated
        let is_defender = *team == Team::Defenders;
        let disable_flocking = is_defender && !defenders_activated.active;
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
    mut all_units: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &Effectiveness,
            Option<&DamageMultiplier>,
            Option<&CauldronDamageBonus>,
            Option<&EliteDamageBonus>,
        ),
        Without<Corpse>,
    >,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&CauldronDamageResistance>,
    )>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - APPROX_FRAME_TIME).max(0.0);

    // Collect snapshot of all units for enemy detection
    let units_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, hitbox, team, _, _, _, _, _)| {
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
        cauldron_damage_bonus,
        elite_damage_bonus,
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
                        _ => team != attacker_team,
                    }
            })
            .filter_map(|(entity, target_pos, target_hitbox, _)| {
                // Calculate distance on XZ plane only (ignore Y axis for attack range)
                let dx = attacker_transform.translation.x - target_pos.x;
                let dz = attacker_transform.translation.z - target_pos.z;
                let distance = (dx * dx + dz * dz).sqrt();
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
                && let Ok((mut target_health, mut temp_hp, target_resistance)) =
                    health_query.get_mut(*target_entity)
            {
                // Apply effectiveness and damage percentage
                // DamageMultiplier stores percentage bonus (0.5 = +50%, 1.0 = +100%)
                // Convert to multiplier: damage * (1.0 + percentage)
                let damage_percentage = damage_mult.map_or(0.0, |d| d.0)
                    + cauldron_damage_bonus.map_or(0.0, |b| b.0)
                    + elite_damage_bonus.map_or(0.0, |b| b.0);
                let damage_multiplier = 1.0 + damage_percentage;
                let mut modified_damage =
                    ATTACK_DAMAGE * effectiveness.multiplier() * damage_multiplier;
                // Apply target's damage resistance (Wormwood brew)
                if let Some(resistance) = target_resistance {
                    modified_damage *= 1.0 - resistance.0;
                }
                apply_damage_to_unit(&mut target_health, temp_hp.as_deref_mut(), modified_damage);
                attack_timing.record_attack(current_time);
            }
        }
    }
}

/// Converts dead units to corpses instead of despawning them.
///
/// When a unit's health reaches zero, this system replaces the unit's material with
/// a pre-loaded corpse material based on team and converts the unit into a corpse
/// that slows living units walking over it.
/// Also records the kill in the kill statistics resource.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn convert_dead_to_corpses(
    mut commands: Commands,
    mut kill_stats: ResMut<super::resources::KillStats>,
    mut spell_kill_events: MessageWriter<DefenderKilledBySpellMessage>,
    mut enemy_kill_events: MessageWriter<EnemyKilledMessage>,
    mut scorched_earth_events: MessageWriter<ScorchedEarthMessage>,
    mut drop_events: MessageWriter<super::drops::messages::SpawnIngredientDropMessage>,
    query: Query<
        (
            Entity,
            &Health,
            &Team,
            &Transform,
            Option<&Infantry>,
            Option<&Archer>,
            Option<&super::units::king::components::King>,
            Option<&SpellDamaged>,
            Option<&ResidualFireDamaged>,
        ),
        Without<Corpse>,
    >,
    infantry_assets: Res<super::units::infantry::resources::InfantryAssets>,
    archer_assets: Res<super::units::archer::resources::ArcherAssets>,
    king_assets: Res<super::units::king::resources::KingAssets>,
    mut velocity_query: Query<&mut Velocity>,
) {
    for (
        entity,
        health,
        team,
        transform,
        is_infantry,
        is_archer,
        is_king,
        spell_damaged,
        residual_fire_damaged,
    ) in &query
    {
        if health.is_dead() {
            // Record the kill
            kill_stats.record_kill(*team);

            // Send enemy killed message for multi-kill achievements
            if *team == Team::Attackers || *team == Team::Undead {
                enemy_kill_events.write(EnemyKilledMessage);
                // Notify drops system of potential ingredient drop
                drop_events.write(super::drops::messages::SpawnIngredientDropMessage {
                    position: transform.translation,
                });
            }

            // Track spell kills on defenders and king
            if spell_damaged.is_some() {
                if *team == Team::Defenders {
                    kill_stats.record_spell_kill_defender();
                    spell_kill_events.write(DefenderKilledBySpellMessage);
                }

                if is_king.is_some() {
                    kill_stats.record_king_killed_by_spell();
                }
            }

            // Scorched Earth: unit died from residual fire damage
            if residual_fire_damaged.is_some() {
                scorched_earth_events.write(ScorchedEarthMessage);
            }

            // Replace with appropriate corpse material
            let corpse_material = if is_king.is_some() {
                king_assets.corpse_material.clone()
            } else if is_infantry.is_some() {
                match team {
                    Team::Defenders => infantry_assets.defender_corpse_material.clone(),
                    Team::Attackers => infantry_assets.attacker_corpse_material.clone(),
                    Team::Undead => infantry_assets.undead_corpse_material.clone(),
                }
            } else if is_archer.is_some() {
                match team {
                    Team::Defenders => archer_assets.defender_corpse_material.clone(),
                    Team::Attackers => archer_assets.attacker_corpse_material.clone(),
                    Team::Undead => archer_assets.undead_corpse_material.clone(),
                }
            } else {
                // Fallback for other unit types (shouldn't happen but be safe)
                match team {
                    Team::Defenders => infantry_assets.defender_corpse_material.clone(),
                    Team::Attackers => infantry_assets.attacker_corpse_material.clone(),
                    Team::Undead => infantry_assets.undead_corpse_material.clone(),
                }
            };

            commands
                .entity(entity)
                .insert(MeshMaterial3d(corpse_material));

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

            // Keep Velocity and Acceleration so corpses can be affected by external forces (e.g., black hole)
            // But reset velocity to zero so they don't continue moving from their death momentum
            if let Ok(mut velocity) = velocity_query.get_mut(entity) {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }

            entity_commands
                .remove::<MovementSpeed>() // Can't move on their own
                .remove::<AttackTiming>() // Can't attack
                .remove::<Hitbox>() // Remove collision
                .remove::<crate::game::components::Billboard>() // Remove billboard so corpse stays flat
                .remove::<super::units::components::CommanderAuraSpeedModifier>() // Remove speed modifiers
                .remove::<super::units::components::FrostSlowModifier>()
                .remove::<super::units::components::RootedModifier>()
                .remove::<super::units::components::HasteModifier>()
                .remove::<super::units::components::RoughTerrainModifier>()
                .remove::<CauldronDamageBonus>()
                .remove::<CauldronDamageResistance>()
                .remove::<CauldronSpeedModifier>();
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

/// Cleans up game entities when replaying (transitioning from ScoreScreen).
///
/// This system runs on OnExit(InGameState::ScoreScreen) and despawns all game entities
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
    mut units: Query<
        (
            &mut Transform,
            &Hitbox,
            Option<&mut super::components::Velocity>,
            Option<&super::units::components::TargetingVelocity>,
        ),
        Without<Corpse>,
    >,
) {
    for (mut transform, hitbox, mut velocity_opt, targeting_velocity) in &mut units {
        // Get the desired movement direction for intelligent collision response
        let desired_direction = if let Some(vel) = velocity_opt.as_ref() {
            Some(Vec3::new(vel.x, 0.0, vel.z).normalize_or_zero())
        } else {
            targeting_velocity.map(|tv| tv.velocity.normalize_or_zero())
        };

        for wall in &walls {
            if let Some(corrected) =
                wall.push_out(transform.translation, hitbox.radius, desired_direction)
            {
                // Calculate the correction vector
                let correction = Vec3::new(
                    corrected.x - transform.translation.x,
                    0.0,
                    corrected.z - transform.translation.z,
                );

                let correction_magnitude = correction.length();

                // Apply position correction
                transform.translation.x = corrected.x;
                transform.translation.z = corrected.z;

                // Adjust velocity to redirect around the wall with stronger force
                if let Some(ref mut velocity) = velocity_opt {
                    let correction_normal = correction.normalize_or_zero();
                    let velocity_vec = Vec3::new(velocity.x, 0.0, velocity.z);
                    let velocity_magnitude = velocity_vec.length();

                    // Remove velocity component perpendicular to wall, keep tangential
                    let perpendicular_component = velocity_vec.dot(correction_normal);

                    if perpendicular_component < 0.0 {
                        // Project onto wall surface (slide along)
                        let tangent_velocity =
                            velocity_vec - correction_normal * perpendicular_component;

                        // Add extra repulsive force to help units flow around the wall
                        // Stronger when deeper in collision
                        let repulsion_strength = (correction_magnitude / hitbox.radius).min(1.0);
                        let repulsion_force =
                            correction_normal * velocity_magnitude * repulsion_strength * 1.5;

                        let final_velocity = tangent_velocity + repulsion_force;

                        // Clamp to original velocity magnitude to avoid speeding up
                        let final_velocity_normalized = final_velocity.normalize_or_zero();
                        velocity.x = final_velocity_normalized.x * velocity_magnitude;
                        velocity.z = final_velocity_normalized.z * velocity_magnitude;
                    }
                }
            }
        }
    }
}

/// Resets game resources when replaying (transitioning from ScoreScreen).
///
/// This system runs on OnExit(InGameState::ScoreScreen) and resets resources like
/// the attack cycle timer and defender activation status.
pub fn reset_resources_for_replay(
    mut attack_cycle: ResMut<super::plugin::GlobalAttackCycle>,
    mut defenders_activated: ResMut<super::units::infantry::components::DefendersActivated>,
    mut king_spawned: ResMut<KingSpawned>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    mut battle_insight: ResMut<super::resources::BattleInsightData>,
) {
    attack_cycle.current_time = 0.0;
    defenders_activated.active = false;
    king_spawned.0 = false;
    cauldron_buffs.reset();
    *battle_insight = Default::default();
}

/// Activates all defenders when any defender is close enough to an enemy.
///
/// This creates coordinated defensive behavior - the entire defensive line
/// engages together rather than individually.
pub fn activate_defenders_on_proximity(
    mut defenders_activated: ResMut<super::units::infantry::components::DefendersActivated>,
    defenders: Query<(&Transform, &Team), Without<Corpse>>,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    const ENGAGEMENT_RANGE: f32 = 800.0; // Archer max range (700) + 100

    // If already active, stay active (defenders don't deactivate once engaged)
    if defenders_activated.active {
        return;
    }

    // Check if any defender is close to any enemy
    for (defender_transform, defender_team) in &defenders {
        // Skip non-defenders
        if *defender_team != Team::Defenders {
            continue;
        }

        // Check distance to nearest enemy
        for (enemy_transform, enemy_team) in &all_units {
            // Skip same team
            if *enemy_team == *defender_team {
                continue;
            }

            // Check if enemy (using same logic as combat)
            let is_enemy = match (*defender_team, *enemy_team) {
                (Team::Undead, Team::Undead) => false,
                (Team::Undead, _) => true,
                (_, Team::Undead) => true,
                _ => *enemy_team != *defender_team,
            };

            if !is_enemy {
                continue;
            }

            let distance = defender_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < ENGAGEMENT_RANGE {
                // Enemy in range - activate all defenders
                defenders_activated.active = true;
                return;
            }
        }
    }
}
