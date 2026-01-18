use bevy::prelude::*;
use std::collections::HashSet;

use super::components::*;
use super::styles::*;
use crate::game::attacker::components::Attacker;
use crate::game::components::{OnGameplayScreen, Velocity};

/// Spawns defender units periodically.
///
/// Uses a local timer to spawn defenders every SPAWN_INTERVAL seconds.
/// Defenders spawn near the wizard at the castle (foreground).
pub fn spawn_defenders(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    *timer += time.delta_secs();

    if *timer >= SPAWN_INTERVAL {
        *timer = 0.0;

        // Spawn defender as a circle mesh
        let circle = Circle::new(UNIT_RADIUS);

        commands.spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: DEFENDER_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(-1000.0, 50.0, 1000.0), // Near castle in bottom-left
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Defender,
            OnGameplayScreen,
        ));
    }
}

/// Updates defender velocities to target the nearest attacker.
///
/// Defenders move in 3D space toward the closest attacker.
pub fn update_defender_targets(
    mut defenders: Query<(&Transform, &mut Velocity), With<Defender>>,
    attackers: Query<&Transform, With<Attacker>>,
) {
    for (def_transform, mut def_velocity) in &mut defenders {
        if let Some(nearest_attacker) = attackers.iter().min_by(|a, b| {
            let dist_a = def_transform.translation.distance(a.translation);
            let dist_b = def_transform.translation.distance(b.translation);
            dist_a.partial_cmp(&dist_b).unwrap()
        }) {
            let direction =
                (nearest_attacker.translation - def_transform.translation).normalize_or_zero();
            def_velocity.x = direction.x * UNIT_SPEED;
            def_velocity.y = direction.y * UNIT_SPEED;
            def_velocity.z = direction.z * UNIT_SPEED;
        } else {
            // No attackers, stop moving
            def_velocity.x = 0.0;
            def_velocity.y = 0.0;
            def_velocity.z = 0.0;
        }
    }
}

/// Handles combat between defenders and attackers.
///
/// When units are within COMBAT_RANGE of each other, both units are despawned.
/// This is a simple collision-based combat system without health/damage.
pub fn combat(
    mut commands: Commands,
    defenders: Query<(Entity, &Transform), With<Defender>>,
    attackers: Query<(Entity, &Transform), With<Attacker>>,
) {
    let mut to_despawn = HashSet::new();

    for (def_entity, def_transform) in &defenders {
        for (att_entity, att_transform) in &attackers {
            let distance = def_transform
                .translation
                .distance(att_transform.translation);

            if distance < COMBAT_RANGE {
                to_despawn.insert(def_entity);
                to_despawn.insert(att_entity);
            }
        }
    }

    // Despawn all units that collided
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}
