use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::RayAssets;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::seeded_rng::resources::GameRng;

pub fn spawn_ray_stalk_particles(
    time: Res<Time>,
    mut commands: Commands,
    body_query: Query<&Transform, (With<Ray>, Without<RayEye>)>,
    eyes: Query<(Entity, &Transform, &RayEye), Without<RayEyeDying>>,
    ray_assets: Res<RayAssets>,
    mut game_rng: ResMut<GameRng>,
    mut spawn_timer: Local<f32>,
) {
    let Ok(body_tf) = body_query.single() else {
        return;
    };

    *spawn_timer += time.delta_secs();
    if *spawn_timer < RAY_STALK_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *spawn_timer -= RAY_STALK_PARTICLE_SPAWN_INTERVAL;

    let body_pos = body_tf.translation;
    let rng = &mut game_rng.0;

    for (eye_entity, eye_tf, _) in &eyes {
        let to_eye = eye_tf.translation - body_pos;
        let dir = to_eye.normalize_or_zero();
        let wobble_offset = rng.random::<f32>() * std::f32::consts::TAU;

        // Initial velocity toward the eye with some spread
        let spread_x = rng.random::<f32>() * 0.3 - 0.15;
        let spread_z = rng.random::<f32>() * 0.3 - 0.15;
        let initial_vel = (dir + Vec3::new(spread_x, 0.0, spread_z)).normalize_or_zero()
            * RAY_STALK_PARTICLE_SPEED;

        commands.spawn((
            Mesh3d(ray_assets.particle_mesh.clone()),
            MeshMaterial3d(ray_assets.particle_material.clone()),
            Transform::from_translation(body_pos),
            RayStalkParticle {
                eye_entity,
                velocity: initial_vel,
                time_alive: 0.0,
                wobble_offset,
            },
            Billboard,
            OnGameplayScreen,
        ));
    }
}

pub fn update_ray_stalk_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut RayStalkParticle)>,
    eye_transforms: Query<&Transform, (With<RayEye>, Without<RayStalkParticle>)>,
    mut game_rng: ResMut<GameRng>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, mut particle) in &mut particles {
        particle.time_alive += delta;

        let Ok(eye_tf) = eye_transforms.get(particle.eye_entity) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        let eye_pos = eye_tf.translation;

        let to_eye = eye_pos - transform.translation;
        let dist = to_eye.length();

        if dist < RAY_STALK_PARTICLE_RADIUS * 2.0 || particle.time_alive >= 5.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        let desired = to_eye.normalize_or_zero() * RAY_STALK_PARTICLE_SPEED;
        let steer = (desired - particle.velocity) * (RAY_STALK_PARTICLE_HOMING * delta).min(1.0);
        particle.velocity += steer;

        let speed = particle.velocity.length();
        if speed > RAY_STALK_PARTICLE_SPEED {
            particle.velocity = particle.velocity.normalize() * RAY_STALK_PARTICLE_SPEED;
        }

        // Wobble perpendicular to travel direction
        let t = particle.time_alive * RAY_STALK_PARTICLE_WOBBLE_FREQ + particle.wobble_offset;
        let forward = particle.velocity.normalize_or_zero();
        let right = Vec3::new(-forward.z, 0.0, forward.x);
        let wobble = right * t.sin() * RAY_STALK_PARTICLE_WOBBLE_AMP
            + Vec3::Y * (t * 1.3).cos() * RAY_STALK_PARTICLE_WOBBLE_AMP * 0.5;

        // Random shudder
        let rng = &mut game_rng.0;
        let shudder = Vec3::new(
            (rng.random::<f32>() - 0.5) * 2.0,
            (rng.random::<f32>() - 0.5) * 2.0,
            (rng.random::<f32>() - 0.5) * 2.0,
        ) * RAY_STALK_PARTICLE_SHUDDER;

        transform.translation += (particle.velocity + wobble + shudder) * delta;
    }
}
