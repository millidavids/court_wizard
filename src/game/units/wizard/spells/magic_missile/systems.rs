use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::events::{MouseLeftHeld, MouseLeftReleased};
use crate::game::units::components::{Health, Team, TemporaryHitPoints, apply_damage_to_unit};
use crate::game::units::wizard::components::{CastingState, Mana, PrimedSpell, Spell, Wizard};

/// Handles magic missile casting with left-click.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, enters channeling state where missiles spawn continuously.
/// Only casts when Magic Missile is the primed spell.
#[allow(clippy::too_many_arguments)]
pub fn handle_magic_missile_casting(
    time: Res<Time>,
    mut mouse_left_held: MessageReader<MouseLeftHeld>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell, &Wizard), With<Wizard>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    targets: Query<(Entity, &Transform, &Team), Without<MagicMissile>>,
) {
    let Ok((mut casting_state, mut mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };

    // Only respond to left-click if Magic Missile is primed
    if primed_spell.spell != Spell::MagicMissile {
        return;
    }

    // Check for release event
    if mouse_left_released.read().next().is_some() {
        // Cancel cast/channel on release
        casting_state.cancel();
        return;
    }

    // Check for hold event
    if mouse_left_held.read().next().is_none() {
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
                if mana.consume(constants::MANA_COST) {
                    spawn_magic_missile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &camera_query,
                        &targets,
                        wizard.spell_range,
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
                if mana.consume(constants::MANA_COST) {
                    spawn_magic_missile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &camera_query,
                        &targets,
                        wizard.spell_range,
                    );
                    casting_state.start_channeling();
                } else {
                    // Out of mana - cancel cast
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            // Not casting or channeling - start new cast
            casting_state.start_cast();
        }
    }
}

/// Spawns a single magic missile projectile.
///
/// Helper function for spawning missiles with random trajectories that arc towards camera.
/// Selects a random target within spell range, or falls back to closest target.
fn spawn_magic_missile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_query: &Query<&GlobalTransform, With<Camera>>,
    targets: &Query<(Entity, &Transform, &Team), Without<MagicMissile>>,
    spell_range: f32,
) {
    // Spawn position: above the wizard
    let spawn_pos = WIZARD_POSITION + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

    // Select target: random attacker within range, or closest attacker
    let mut rng = rand::thread_rng();

    let attackers_in_range: Vec<Entity> = targets
        .iter()
        .filter(|(_, _, team)| **team == Team::Attackers)
        .filter(|(_, transform, _)| {
            let distance = spawn_pos.distance(transform.translation);
            distance <= spell_range
        })
        .map(|(entity, _, _)| entity)
        .collect();

    let target = if !attackers_in_range.is_empty() {
        // Pick a random target within range
        let index = rng.gen_range(0..attackers_in_range.len());
        Some(attackers_in_range[index])
    } else {
        // No targets in range, find the closest attacker anywhere
        targets
            .iter()
            .filter(|(_, _, team)| **team == Team::Attackers)
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
        MagicMissile::new(initial_velocity, wobble_offset, target),
        OnGameplayScreen,
    ));
}

/// Updates magic missile movement with homing and wobble.
///
/// Missiles lock onto their initial target and only retarget if it despawns.
pub fn move_magic_missiles(
    time: Res<Time>,
    mut missiles: Query<(&mut Transform, &mut MagicMissile)>,
    targets: Query<(Entity, &Transform, &Team), Without<MagicMissile>>,
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
            // Select new target: random attacker within range, or closest attacker
            let mut rng = rand::thread_rng();

            let attackers_in_range: Vec<Entity> = targets
                .iter()
                .filter(|(_, _, team)| **team == Team::Attackers)
                .filter(|(_, transform, _)| {
                    let distance = missile_transform
                        .translation
                        .distance(transform.translation);
                    distance <= spell_range
                })
                .map(|(entity, _, _)| entity)
                .collect();

            missile.target = if !attackers_in_range.is_empty() {
                // Pick a random target within range
                let index = rng.gen_range(0..attackers_in_range.len());
                Some(attackers_in_range[index])
            } else {
                // No targets in range, find the closest attacker anywhere
                targets
                    .iter()
                    .filter(|(_, _, team)| **team == Team::Attackers)
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
            // No attackers left, just continue with current velocity
            missile_transform.translation += missile.velocity * time.delta_secs();
        }
    }
}

/// Checks for magic missile collisions with attackers.
///
/// When a missile hits an attacker, it deals 50 damage and despawns.
pub fn check_magic_missile_collisions(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform, &MagicMissile)>,
    mut attackers: Query<
        (
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
        ),
        Without<MagicMissile>,
    >,
) {
    for (missile_entity, missile_transform, missile) in &missiles {
        for (attacker_transform, mut health, mut temp_hp, team) in &mut attackers {
            // Only damage attackers
            if *team != Team::Attackers {
                continue;
            }

            let distance = missile_transform
                .translation
                .distance(attacker_transform.translation);

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
