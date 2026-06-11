use bevy::prelude::*;
use rand::Rng;

use crate::game::components::OnGameplayScreen;

/// Seconds between spawning a new animated ring particle.
pub(crate) const RING_SPAWN_INTERVAL: f32 = 0.08;
/// Lifetime of each animated ring (seconds).
const RING_LIFETIME: f32 = 2.4;
/// Minimum max-scale for animated rings.
const RING_MIN_SCALE: f32 = 8.0;
/// Maximum max-scale for animated rings.
const RING_MAX_SCALE: f32 = 28.0;
/// Maximum tilt from horizontal for animated rings (radians).
const RING_MAX_TILT: f32 = 1.0;

/// Animated ring particle that grows outward and shrinks back.
/// Shared by entangle vine rings and spike growth vine/spike rings.
#[derive(Component)]
pub(crate) struct AnimatedRingParticle {
    pub time_alive: f32,
    pub lifetime: f32,
    pub max_scale: f32,
}

/// Spawns a single animated ring particle at a random position within a circle.
pub(crate) fn spawn_ring_particle(
    rng: &mut impl Rng,
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    center: Vec3,
    radius: f32,
) {
    let angle = rng.random::<f32>() * std::f32::consts::TAU;
    let dist = radius * rng.random::<f32>().sqrt() * 0.9;
    let x = center.x + angle.cos() * dist;
    let z = center.z + angle.sin() * dist;

    let yaw = rng.random::<f32>() * std::f32::consts::TAU;
    let tilt = 0.3 + rng.random::<f32>() * RING_MAX_TILT;
    let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(tilt);

    let max_scale = RING_MIN_SCALE + rng.random::<f32>() * (RING_MAX_SCALE - RING_MIN_SCALE);
    let y = -max_scale * 0.3 * tilt.sin();
    let lifetime = RING_LIFETIME * (0.7 + rng.random::<f32>() * 0.6);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(x, y, z))
            .with_rotation(rotation)
            .with_scale(Vec3::ZERO),
        AnimatedRingParticle {
            time_alive: 0.0,
            lifetime,
            max_scale,
        },
        OnGameplayScreen,
    ));
}

/// Animates ring particles: grow outward then shrink, despawn when lifetime expires.
pub(crate) fn animate_ring_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut rings: Query<(Entity, &mut AnimatedRingParticle, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut ring, mut transform) in &mut rings {
        ring.time_alive += delta;
        let progress = (ring.time_alive / ring.lifetime).clamp(0.0, 1.0);

        // Grow in first 40% (ease-out), hold 40-60%, shrink last 40% (ease-in)
        let scale_fraction = if progress < 0.4 {
            let t = progress / 0.4;
            1.0 - (1.0 - t) * (1.0 - t)
        } else if progress < 0.6 {
            1.0
        } else {
            let t = (progress - 0.6) / 0.4;
            (1.0 - t) * (1.0 - t)
        };
        transform.scale = Vec3::splat(ring.max_scale * scale_fraction);

        if ring.time_alive >= ring.lifetime {
            commands.entity(entity).try_despawn();
        }
    }
}
