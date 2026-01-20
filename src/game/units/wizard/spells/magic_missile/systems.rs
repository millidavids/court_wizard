use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::units::components::{Health, Team};
use crate::game::units::infantry::components::Infantry;
use crate::game::units::wizard::components::{Mana, Wizard};

/// Mana cost for casting a magic missile.
const MAGIC_MISSILE_MANA_COST: f32 = 10.0;

/// Casts a magic missile when spacebar is pressed.
///
/// Spawns a pink projectile above the wizard with varied launch trajectory.
/// Requires and consumes mana.
pub fn cast_magic_missile(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<&mut Mana, With<Wizard>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // Check if wizard has enough mana
    let Ok(mut mana) = wizard_query.single_mut() else {
        return;
    };

    if !mana.consume(MAGIC_MISSILE_MANA_COST) {
        // Not enough mana
        return;
    }

    // Spawn position: above the wizard
    let spawn_pos = WIZARD_POSITION + Vec3::new(0.0, 100.0, 0.0);

    // Random initial velocity: varied launch paths (up and to the sides, never down)
    let mut rng = rand::thread_rng();
    let horizontal_x = rng.gen_range(-200.0..200.0);
    let horizontal_z = rng.gen_range(-200.0..200.0);
    let vertical = rng.gen_range(300.0..500.0); // Always upward
    let initial_velocity = Vec3::new(horizontal_x, vertical, horizontal_z);

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
        MagicMissile::new(initial_velocity, wobble_offset),
        OnGameplayScreen,
    ));
}

/// Updates magic missile movement with homing and wobble.
///
/// Missiles always seek the closest attacker each frame.
pub fn move_magic_missiles(
    time: Res<Time>,
    mut missiles: Query<(&mut Transform, &mut MagicMissile)>,
    targets: Query<(&Transform, &Team), (With<Infantry>, Without<MagicMissile>)>,
) {
    for (mut missile_transform, mut missile) in &mut missiles {
        missile.time_alive += time.delta_secs();

        // Find nearest attacker each frame
        let nearest_target = targets
            .iter()
            .filter(|(_, team)| **team == Team::Attackers)
            .min_by(|a, b| {
                let dist_a = missile_transform.translation.distance(a.0.translation);
                let dist_b = missile_transform.translation.distance(b.0.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some((target_transform, _)) = nearest_target {
            let to_target = target_transform.translation - missile_transform.translation;
            let distance_to_target = to_target.length();
            let current_homing_strength = missile.current_homing_strength();

            // Calculate proximity-based speed (slow down near target to avoid overshooting)
            let base_max_speed = missile.current_max_speed();
            let min_speed = 300.0; // 1.5x infantry movement speed (200 * 1.5)
            let slowdown_distance = 300.0; // Start slowing within this distance

            let proximity_speed_multiplier = if distance_to_target < slowdown_distance {
                // Linearly interpolate from 1.0 (far) to min_speed/base_max_speed (near)
                let t = (distance_to_target / slowdown_distance).clamp(0.0, 1.0);
                let min_multiplier = min_speed / base_max_speed;
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
            let wobble = if missile.time_alive < 5.0 {
                let wobble_freq = 3.0;
                let wobble_amplitude = 30.0;
                let t = missile.time_alive * wobble_freq + missile.wobble_offset;

                Vec3::new(
                    t.sin() * wobble_amplitude,
                    (t * 1.3).cos() * wobble_amplitude * 0.5, // Less vertical wobble
                    (t * 0.7).sin() * wobble_amplitude,
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
    mut attackers: Query<(&Transform, &mut Health, &Team), With<Infantry>>,
) {
    for (missile_entity, missile_transform, missile) in &missiles {
        for (attacker_transform, mut health, team) in &mut attackers {
            // Only damage attackers
            if *team != Team::Attackers {
                continue;
            }

            let distance = missile_transform
                .translation
                .distance(attacker_transform.translation);

            // Check collision
            if distance < missile.radius {
                health.take_damage(missile.damage);
                commands.entity(missile_entity).despawn();
                break; // Missile destroyed, stop checking
            }
        }
    }
}

/// Despawns magic missiles that are too far from the battlefield.
pub fn despawn_distant_magic_missiles(
    mut commands: Commands,
    missiles: Query<(Entity, &Transform), With<MagicMissile>>,
) {
    const MAX_DISTANCE: f32 = 10000.0;

    for (entity, transform) in &missiles {
        let distance_from_origin = transform.translation.length();

        if distance_from_origin > MAX_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}
