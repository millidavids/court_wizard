use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::super::super::components::{CastingState, GuestWizard, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput};
use super::components::*;
use super::constants;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState, SkipSpellSpawning};
use crate::networking::protocol::{NetworkMessage, SpellAction};
use crate::networking::resources::NetworkConnection;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed its initial cast (entered channeling).
    completed: bool,
    /// Whether the spell is actively channeling this frame.
    channeling: bool,
    /// Whether the channel was just cancelled this frame.
    channel_ended: bool,
}

/// Local wizard magic missile casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_magic_missile_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (&Transform, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(Entity, &Transform, &ArcaneCrystal)>,
    skip_spawning: Option<Res<SkipSpellSpawning>>,
    mut connection: Option<ResMut<NetworkConnection>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query_3d, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_transform, mut casting_state, mut mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::MagicMissile { return; }

    let skip_spawn = skip_spawning.is_some();

    let cast_result = magic_missile_casting_logic(
        &input,
        &time,
        wizard_transform,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        TargetTeams::AttackersAndUndead,
        &mut commands,
        &mut meshes,
        &mut materials,
        &camera_query,
        &targets,
        &crystals,
        skip_spawn,
    );

    if cast_result.completed {
        mouse_state.left_consumed = true;

        // Guest: notify host that channeling started
        if skip_spawn {
            if let Some(conn) = connection.as_mut() {
                let pos = cursor_pos.unwrap_or(Vec3::ZERO);
                conn.outgoing_messages.push(NetworkMessage::SpellResult(
                    SpellAction::ChannelStarted {
                        spell: Spell::MagicMissile,
                        cursor_pos: [pos.x, pos.y, pos.z],
                        empowerment: primed_spell.empowerment,
                    },
                ));
            }
        }
    }

    // Guest: send channel updates each frame during channeling
    if skip_spawn && cast_result.channeling {
        if let Some(conn) = connection.as_mut() {
            let pos = cursor_pos.unwrap_or(Vec3::ZERO);
            conn.outgoing_messages.push(NetworkMessage::SpellResult(
                SpellAction::ChannelUpdate {
                    cursor_pos: [pos.x, pos.y, pos.z],
                },
            ));
        }
    }

    // Guest: notify host that channeling ended
    if skip_spawn && cast_result.channel_ended {
        if let Some(conn) = connection.as_mut() {
            conn.outgoing_messages.push(NetworkMessage::SpellResult(
                SpellAction::ChannelEnded,
            ));
        }
    }
}

/// Guest wizard magic missile casting — reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_magic_missile_casting_guest(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (&Transform, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<GuestWizard>,
    >,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    targets: Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: Query<(Entity, &Transform, &ArcaneCrystal)>,
    guest_cursor: Res<GuestCursorPosition>,
    guest_input: Res<GuestInputState>,
) {
    let input = WizardInput {
        just_pressed: guest_input.just_pressed,
        pressed: guest_input.pressed,
        just_released: guest_input.just_released,
        cursor_pos: guest_cursor.position,
    };

    let Ok((wizard_transform, mut casting_state, mut mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::MagicMissile { return; }

    magic_missile_casting_logic(
        &input,
        &time,
        wizard_transform,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        TargetTeams::DefendersAndUndead,
        &mut commands,
        &mut meshes,
        &mut materials,
        &camera_query,
        &targets,
        &crystals,
        false, // Host always spawns
    );
}

/// Core magic missile casting logic — called by both local and guest systems.
///
/// Handles the full Resting -> Casting -> Channeling state machine.
/// During channeling, spawns missiles at increasing frequency.
#[allow(clippy::too_many_arguments)]
fn magic_missile_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_transform: &Transform,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    target_teams: TargetTeams,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_query: &Query<&GlobalTransform, With<Camera>>,
    targets: &Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: &Query<(Entity, &Transform, &ArcaneCrystal)>,
    skip_spawn: bool,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        channeling: false,
        channel_ended: false,
    };

    let spawn_origin = wizard_transform.translation;

    // Check for release event
    if input.just_released {
        let was_channeling = matches!(*casting_state, CastingState::Channeling { .. });
        casting_state.cancel();
        if was_channeling {
            result.channel_ended = true;
        }
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());
            result.channeling = true;

            if casting_state.should_channel(
                constants::INITIAL_CHANNEL_INTERVAL,
                constants::MIN_CHANNEL_INTERVAL,
                constants::CHANNEL_RAMP_TIME,
            ) {
                let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
                if mana.consume(mana_cost) {
                    if !skip_spawn {
                        spawn_magic_missile(
                            commands,
                            meshes,
                            materials,
                            camera_query,
                            targets,
                            crystals,
                            wizard.spell_range,
                            primed_spell.empowerment,
                            input.cursor_pos,
                            spawn_origin,
                            target_teams,
                        );
                    }
                    casting_state.reset_channel_interval();
                } else {
                    casting_state.cancel();
                    result.channeling = false;
                    result.channel_ended = true;
                }
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
                if mana.consume(mana_cost) {
                    if !skip_spawn {
                        spawn_magic_missile(
                            commands,
                            meshes,
                            materials,
                            camera_query,
                            targets,
                            crystals,
                            wizard.spell_range,
                            primed_spell.empowerment,
                            input.cursor_pos,
                            spawn_origin,
                            target_teams,
                        );
                    }
                    casting_state.start_channeling();
                    result.completed = true;
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            let mana_cost = constants::MANA_COST * wizard.mana_cost_multiplier;
            if (input.just_pressed || input.pressed) && mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
    }

    result
}

/// Spawns a single magic missile projectile.
///
/// Helper function for spawning missiles with random trajectories that arc towards camera.
/// If cursor position is provided, preferentially targets enemies near cursor using weighted random selection.
/// Falls back to closest target if no enemies are in range.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_magic_missile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_query: &Query<&GlobalTransform, With<Camera>>,
    targets: &Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: &Query<(Entity, &Transform, &ArcaneCrystal)>,
    spell_range: f32,
    empowerment: f32,
    cursor_world_pos: Option<Vec3>,
    spawn_origin: Vec3,
    target_teams: TargetTeams,
) {
    // Spawn position: above the wizard
    let spawn_pos = spawn_origin + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

    // Priority: if cursor is near an Arcane Crystal, target it directly
    let crystal_target: Option<Entity> = cursor_world_pos.and_then(|cursor_pos| {
        let mut closest: Option<(Entity, f32)> = None;
        for (entity, transform, crystal) in crystals.iter() {
            let dist = Vec3::new(
                cursor_pos.x - transform.translation.x,
                0.0,
                cursor_pos.z - transform.translation.z,
            )
            .length();
            if dist <= crystal.collision_radius * 5.0 {
                match closest {
                    None => closest = Some((entity, dist)),
                    Some((_, prev_dist)) if dist < prev_dist => closest = Some((entity, dist)),
                    _ => {}
                }
            }
        }
        closest.map(|(e, _)| e)
    });

    // Select target using cursor-based weighted selection if cursor position is available
    let mut rng = rand::thread_rng();

    let target = if crystal_target.is_some() {
        crystal_target
    } else {
        // Collect enemies in range
        let enemies_in_range: Vec<Entity> = targets
            .iter()
            .filter(|(_, _, team)| target_teams.matches(team))
            .filter(|(_, transform, _)| {
                let distance = spawn_pos.distance(transform.translation);
                distance <= spell_range
            })
            .map(|(entity, _, _)| entity)
            .collect();

        if !enemies_in_range.is_empty() {
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
                            let weight = 1.0
                                / (distance.powi(constants::CURSOR_TARGETING_WEIGHT_POWER) + 1.0);
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
                .filter(|(_, _, team)| target_teams.matches(team))
                .min_by(|a, b| {
                    let dist_a = spawn_pos.distance(a.1.translation);
                    let dist_b = spawn_pos.distance(b.1.translation);
                    dist_a.partial_cmp(&dist_b).unwrap()
                })
                .map(|(entity, _, _)| entity)
        }
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
        MagicMissile::new(
            initial_velocity,
            wobble_offset,
            target,
            empowerment,
            target_teams,
            spell_range,
            spawn_pos,
        ),
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
    crystal_transforms: Query<&Transform, (With<ArcaneCrystal>, Without<MagicMissile>)>,
) {
    for (mut missile_transform, mut missile) in &mut missiles {
        missile.time_alive += time.delta_secs();

        // Check if current target still exists (could be a unit or a crystal)
        let target_exists = missile.target.is_some_and(|target_entity| {
            targets.get(target_entity).is_ok() || crystal_transforms.get(target_entity).is_ok()
        });

        // Retarget if current target despawned
        if !target_exists {
            // Select new target: random enemy (Attacker or Undead) within range, or closest enemy
            let mut rng = rand::thread_rng();

            let enemies_in_range: Vec<Entity> = targets
                .iter()
                .filter(|(_, _, team)| missile.target_teams.matches(team))
                .filter(|(_, transform, _)| {
                    let distance = missile_transform
                        .translation
                        .distance(transform.translation);
                    distance <= missile.spell_range
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
                    .filter(|(_, _, team)| missile.target_teams.matches(team))
                    .min_by(|a, b| {
                        let dist_a = missile_transform.translation.distance(a.1.translation);
                        let dist_b = missile_transform.translation.distance(b.1.translation);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(entity, _, _)| entity)
            };
        }

        // Get current target's transform (check units first, then crystals)
        let target_transform = missile.target.and_then(|target_entity| {
            targets
                .get(target_entity)
                .ok()
                .map(|(_, transform, _)| transform)
                .or_else(|| crystal_transforms.get(target_entity).ok())
        });

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
            if !missile.target_teams.matches(team) {
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

/// Despawns magic missiles that exit their spell range from their origin.
pub fn despawn_distant_magic_missiles(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform, &MagicMissile)>,
) {
    for (entity, transform, missile) in &missiles {
        let distance_from_origin = transform.translation.distance(missile.origin_pos);
        if distance_from_origin > missile.spell_range {
            commands.entity(entity).despawn();
        }
    }
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
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
