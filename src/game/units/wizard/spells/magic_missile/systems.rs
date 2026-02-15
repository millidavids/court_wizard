use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Wizard};
use super::components::*;
use super::constants;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Handles magic missile casting with left-click.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, enters channeling state where missiles spawn continuously.
/// Only casts when Magic Missile is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_magic_missile_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell, &Wizard), With<Wizard>>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
) {
    let Ok((mut casting_state, mut mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if mouse_left_released.read().next().is_some() {
        // Cancel cast/channel on release
        casting_state.cancel();
        return;
    }

    // Mouse is held - handle casting or channeling based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Already channeling - advance channel time
            casting_state.advance_channel(time.delta_secs());

            // Check if enough time has passed to spawn another missile
            if casting_state.should_channel(
                constants::INITIAL_CHANNEL_INTERVAL,
                constants::MIN_CHANNEL_INTERVAL,
                constants::CHANNEL_RAMP_TIME,
            ) {
                // Try to spawn missile if we have mana
                let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
                if mana.consume(mana_cost) {
                    let cursor_pos = get_cursor_world_position(&camera_query_3d, &window_query);
                    spawn_magic_missile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &camera_query,
                        &targets,
                        wizard.spell_range,
                        primed_spell.empowerment,
                        cursor_pos,
                    );
                    casting_state.reset_channel_interval();
                } else {
                    // Out of mana - cancel channeling
                    casting_state.cancel();
                }
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - transition to channeling and spawn first missile
                let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
                if mana.consume(mana_cost) {
                    let cursor_pos = get_cursor_world_position(&camera_query_3d, &window_query);
                    spawn_magic_missile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &camera_query,
                        &targets,
                        wizard.spell_range,
                        primed_spell.empowerment,
                        cursor_pos,
                    );
                    casting_state.start_channeling();
                } else {
                    // Out of mana - cancel cast
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            // Not casting or channeling - check mana before starting cast
            let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
            if mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
    }
}

/// Spawns a single magic missile projectile.
///
/// Helper function for spawning missiles with random trajectories that arc towards camera.
/// If cursor position is provided, preferentially targets enemies near cursor using weighted random selection.
/// Falls back to closest target if no enemies are in range.
#[allow(clippy::too_many_arguments)]
fn spawn_magic_missile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_query: &Query<&GlobalTransform, With<Camera>>,
    targets: &Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    spell_range: f32,
    empowerment: f32,
    cursor_world_pos: Option<Vec3>,
) {
    // Spawn position: above the wizard
    let spawn_pos = WIZARD_POSITION + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

    // Select target using cursor-based weighted selection if cursor position is available
    let mut rng = rand::thread_rng();

    // Collect enemies in range
    let enemies_in_range: Vec<Entity> = targets
        .iter()
        .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
        .filter(|(_, transform, _)| {
            let distance = spawn_pos.distance(transform.translation);
            distance <= spell_range
        })
        .map(|(entity, _, _)| entity)
        .collect();

    let target = if !enemies_in_range.is_empty() {
        if let Some(cursor_pos) = cursor_world_pos {
            // Weighted random selection based on distance from cursor
            // Build weights and total in a single pass without intermediate Vec
            let mut total_weight = 0.0;
            let weighted_targets: Vec<(Entity, f32)> = enemies_in_range
                .iter()
                .filter_map(|&entity| {
                    targets.get(entity).ok().map(|(_, transform, _)| {
                        let distance = cursor_pos.distance(transform.translation);
                        // Inverse distance squared weighting (add 1.0 to avoid division by zero)
                        let weight =
                            1.0 / (distance.powi(constants::CURSOR_TARGETING_WEIGHT_POWER) + 1.0);
                        total_weight += weight;
                        (entity, weight)
                    })
                })
                .collect();

            if total_weight > 0.0 {
                // Pick target using weighted random selection
                let mut random_value = rng.gen_range(0.0..total_weight);
                let mut selected_target = None;
                for (entity, weight) in weighted_targets {
                    random_value -= weight;
                    if random_value <= 0.0 {
                        selected_target = Some(entity);
                        break;
                    }
                }
                // Fallback to first target if loop completes (shouldn't happen)
                selected_target.or_else(|| enemies_in_range.first().copied())
            } else {
                // All weights are zero (shouldn't happen), pick random
                let index = rng.gen_range(0..enemies_in_range.len());
                Some(enemies_in_range[index])
            }
        } else {
            // No cursor position, pick random target within range
            let index = rng.gen_range(0..enemies_in_range.len());
            Some(enemies_in_range[index])
        }
    } else {
        // No targets in range, find the closest enemy anywhere
        targets
            .iter()
            .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
            .min_by(|a, b| {
                let dist_a = spawn_pos.distance(a.1.translation);
                let dist_b = spawn_pos.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(entity, _, _)| entity)
    };

    // Random initial velocity: varied launch paths (up and to the sides, never down)
    let horizontal_x = rng.gen_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let horizontal_z = rng.gen_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let vertical = rng.gen_range(constants::VERTICAL_VEL_MIN..constants::VERTICAL_VEL_MAX);
    let mut initial_velocity = Vec3::new(horizontal_x, vertical, horizontal_z);

    // Add arc towards camera (so sprites appear to grow before arcing down)
    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation();
        let to_camera = (camera_pos - spawn_pos).normalize_or_zero();
        let camera_arc_speed =
            rng.gen_range(constants::CAMERA_ARC_SPEED_MIN..constants::CAMERA_ARC_SPEED_MAX);
        let camera_arc = to_camera * camera_arc_speed;
        initial_velocity += camera_arc;
    }

    // Random wobble offset for this missile
    let wobble_offset = rng.gen_range(0.0..std::f32::consts::TAU);

    // Spawn magic missile as a small pink circle
    let circle = Circle::new(MAGIC_MISSILE_RADIUS);

    commands.spawn((
        Mesh3d(meshes.add(circle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: MAGIC_MISSILE_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(spawn_pos),
        MagicMissile::new(initial_velocity, wobble_offset, target, empowerment),
        OnGameplayScreen,
    ));
}

/// Updates magic missile movement with homing and wobble.
///
/// Missiles lock onto their initial target and only retarget if it despawns.
pub fn move_magic_missiles(
    time: Res<Time>,
    mut missiles: Query<(&mut Transform, &mut MagicMissile)>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    wizard_query: Query<&Wizard>,
) {
    let Ok(wizard) = wizard_query.single() else {
        return;
    };
    let spell_range = wizard.spell_range;

    for (mut missile_transform, mut missile) in &mut missiles {
        missile.time_alive += time.delta_secs();

        // Check if current target still exists
        let target_exists = missile
            .target
            .and_then(|target_entity| targets.get(target_entity).ok())
            .is_some();

        // Retarget if current target despawned
        if !target_exists {
            // Select new target: random enemy (Attacker or Undead) within range, or closest enemy
            let mut rng = rand::thread_rng();

            let enemies_in_range: Vec<Entity> = targets
                .iter()
                .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
                .filter(|(_, transform, _)| {
                    let distance = missile_transform
                        .translation
                        .distance(transform.translation);
                    distance <= spell_range
                })
                .map(|(entity, _, _)| entity)
                .collect();

            missile.target = if !enemies_in_range.is_empty() {
                // Pick a random target within range
                let index = rng.gen_range(0..enemies_in_range.len());
                Some(enemies_in_range[index])
            } else {
                // No targets in range, find the closest enemy anywhere
                targets
                    .iter()
                    .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
                    .min_by(|a, b| {
                        let dist_a = missile_transform.translation.distance(a.1.translation);
                        let dist_b = missile_transform.translation.distance(b.1.translation);
                        dist_a.partial_cmp(&dist_b).unwrap()
                    })
                    .map(|(entity, _, _)| entity)
            };
        }

        // Get current target's transform
        let target_transform = missile
            .target
            .and_then(|target_entity| targets.get(target_entity).ok())
            .map(|(_, transform, _)| transform);

        if let Some(target_transform) = target_transform {
            let to_target = target_transform.translation - missile_transform.translation;
            let distance_to_target = to_target.length();
            let current_homing_strength = missile.current_homing_strength();

            // Calculate proximity-based speed (slow down near target to avoid overshooting)
            let base_max_speed = missile.current_max_speed();

            let proximity_speed_multiplier = if distance_to_target < constants::SLOWDOWN_DISTANCE {
                // Linearly interpolate from 1.0 (far) to min_speed/base_max_speed (near)
                let t = (distance_to_target / constants::SLOWDOWN_DISTANCE).clamp(0.0, 1.0);
                let min_multiplier = constants::MIN_PROXIMITY_SPEED / base_max_speed;
                min_multiplier + (1.0 - min_multiplier) * t
            } else {
                1.0 // Full speed when far from target
            };

            let max_speed = base_max_speed * proximity_speed_multiplier;

            // Calculate homing force (handle perfect tracking)
            let homing_force = if current_homing_strength.is_infinite() {
                // Perfect tracking: move directly toward target center with no momentum
                // Just set direction, speed will be applied based on proximity
                to_target.normalize_or_zero()
            } else {
                // Normal homing with increasing strength
                to_target.normalize_or_zero() * current_homing_strength
            };

            // Add wobble for variation (sine wave in multiple directions)
            // Only apply wobble before perfect tracking kicks in
            let wobble = if missile.time_alive < constants::PERFECT_TRACKING_TIME {
                let t = missile.time_alive * constants::WOBBLE_FREQUENCY + missile.wobble_offset;

                Vec3::new(
                    t.sin() * constants::WOBBLE_AMPLITUDE,
                    (t * constants::WOBBLE_Y_FREQ_MULTIPLIER).cos()
                        * constants::WOBBLE_AMPLITUDE
                        * constants::WOBBLE_Y_AMPLITUDE_MULTIPLIER,
                    (t * constants::WOBBLE_Z_FREQ_MULTIPLIER).sin() * constants::WOBBLE_AMPLITUDE,
                )
            } else {
                Vec3::ZERO // No wobble during perfect tracking
            };

            // Update velocity
            if current_homing_strength.is_infinite() {
                // Perfect tracking: directly set velocity toward target (no momentum)
                missile.velocity = homing_force * max_speed;
            } else {
                // Normal homing: add force to velocity with wobble
                missile.velocity += (homing_force + wobble) * time.delta_secs();

                // Limit speed (increases over time, decreases near target)
                let current_speed = missile.velocity.length();
                if current_speed > max_speed {
                    missile.velocity = missile.velocity.normalize() * max_speed;
                }
            }

            // Apply velocity to position
            missile_transform.translation += missile.velocity * time.delta_secs();
        } else {
            // No enemies left, just continue with current velocity
            missile_transform.translation += missile.velocity * time.delta_secs();
        }
    }
}

/// Checks for magic missile collisions with enemies (Attackers and Undead).
///
/// When a missile hits an enemy, it deals 50 damage and despawns.
pub fn check_magic_missile_collisions(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform, &MagicMissile)>,
    mut enemies: Query<
        (
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
        ),
        (Without<MagicMissile>, Without<Corpse>),
    >,
    walls: Query<&WallOfStone>,
) {
    for (missile_entity, missile_transform, missile) in &missiles {
        // Wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(missile_transform.translation)
                && missile_transform.translation.y <= wall.height
            {
                commands.entity(missile_entity).despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        for (enemy_transform, mut health, mut temp_hp, team) in &mut enemies {
            // Magic Missile targets Attackers and Undead
            if *team != Team::Attackers && *team != Team::Undead {
                continue;
            }

            let distance = missile_transform
                .translation
                .distance(enemy_transform.translation);

            // Check collision
            if distance < missile.radius {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), missile.damage);
                commands.entity(missile_entity).despawn();
                break; // Missile destroyed, stop checking
            }
        }
    }
}

/// Despawns magic missiles that exit the wizard's spell range.
pub fn despawn_distant_magic_missiles(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform), With<MagicMissile>>,
    wizard_query: Query<(&Transform, &Wizard), Without<MagicMissile>>,
) {
    // Get wizard position and spell range
    let Ok((wizard_transform, wizard)) = wizard_query.single() else {
        return;
    };

    let wizard_pos = wizard_transform.translation;
    let spell_range = wizard.spell_range;

    for (entity, transform) in &missiles {
        let distance_from_wizard = transform.translation.distance(wizard_pos);

        if distance_from_wizard > spell_range {
            commands.entity(entity).despawn();
        }
    }
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
///
/// TODO: Extract this to shared wizard utilities - duplicated across 11 spells.
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}
