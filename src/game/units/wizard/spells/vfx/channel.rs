//! Shared "growing sphere → inward implosion" particle VFX used by units that
//! channel their abilities (teleporter, healer, shielder, dispeller).

use bevy::prelude::*;
use rand::Rng;

use crate::game::components::OnGameplayScreen;

#[derive(Component)]
pub(in crate::game) struct ChannelingCast {
    pub elapsed: f32,
}

#[derive(Component)]
pub(in crate::game) struct ChannelParticle {
    pub velocity: Vec3,
    pub time_alive: f32,
    pub lifetime: f32,
    pub size: f32,
}

#[derive(Clone, Copy)]
pub(in crate::game) struct ChannelParticleSpec {
    pub start_radius: f32,
    pub max_radius: f32,
    pub size: f32,
    pub lifetime: f32,
    pub count_per_spawn: usize,
}

pub(in crate::game) fn spawn_channel_particle_batch(
    commands: &mut Commands,
    center: Vec3,
    progress: f32,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    spec: &ChannelParticleSpec,
    rng: &mut impl Rng,
) {
    let p = progress.clamp(0.0, 1.0);
    let radius = spec.start_radius + (spec.max_radius - spec.start_radius) * p;
    let speed = radius / spec.lifetime;
    for _ in 0..spec.count_per_spawn {
        let theta = rng.gen_range(0.0..std::f32::consts::TAU);
        let z = rng.gen_range(-1.0_f32..1.0);
        let r_xy = (1.0 - z * z).sqrt();
        let dir = Vec3::new(r_xy * theta.cos(), z, r_xy * theta.sin());
        let spawn_pos = center + dir * radius;
        let velocity = -dir * speed;

        commands.spawn((
            ChannelParticle {
                velocity,
                time_alive: 0.0,
                lifetime: spec.lifetime,
                size: spec.size,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(spawn_pos)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(spec.size)),
            OnGameplayScreen,
        ));
    }
}

pub(in crate::game) fn update_channel_particles(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut ChannelParticle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform) in &mut particles {
        particle.time_alive += dt;
        if particle.time_alive >= particle.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }
        transform.translation += particle.velocity * dt;
        let remaining = 1.0 - (particle.time_alive / particle.lifetime);
        transform.scale = Vec3::splat(particle.size * remaining);
    }
}
