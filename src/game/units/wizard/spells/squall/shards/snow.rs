//! Snow particle spawning and animation.

use bevy::prelude::*;
use rand::Rng;

use super::super::components::{SnowParticle, SquallStorm};
use super::super::constants::*;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns swirling snow particles within active storm areas.
pub(crate) fn spawn_snow_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    storms: Query<&SquallStorm, Without<crate::game::multiplayer::components::GhostSpellEffect>>,
) {
    let rng = &mut game_rng.0;
    let time_secs = time.elapsed_secs();

    for storm in storms.iter() {
        // Check spawn interval using elapsed time
        let interval = SNOW_SPAWN_INTERVAL;
        let spawn_check = (time_secs / interval) as u32;
        let prev_check = ((time_secs - time.delta_secs()) / interval) as u32;
        if spawn_check == prev_check {
            continue;
        }

        for i in 0..SNOW_BATCH_SIZE {
            let seed = time_secs * 7.1 + i as f32 * 1.618_034;

            // Random position within storm radius
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(0.0..storm.radius);
            let height = rng.random_range(SNOW_MIN_HEIGHT..SNOW_MAX_HEIGHT);

            let spawn_pos = Vec3::new(
                storm.position.x + angle.cos() * distance,
                height,
                storm.position.z + angle.sin() * distance,
            );

            // Tangential velocity for swirling motion
            let swirl_angle = angle + std::f32::consts::FRAC_PI_2;
            let velocity = Vec3::new(
                swirl_angle.cos() * SNOW_SWIRL_SPEED,
                -SNOW_DRIFT_SPEED,
                swirl_angle.sin() * SNOW_SWIRL_SPEED,
            );

            let phase = seed * std::f32::consts::PI + (seed * 41.7).sin();
            let lifetime = SNOW_LIFETIME * rng.random_range(0.7..1.3);
            let base_size = SNOW_BASE_SIZE * rng.random_range(0.5..1.5);

            commands.spawn((
                SnowParticle {
                    velocity,
                    time_alive: 0.0,
                    lifetime,
                    base_size,
                    phase,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.snow_particle.clone()),
                Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.1)),
                Billboard,
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates snow particles: swirling motion, sway, and fade in/out via scale.
pub(crate) fn update_snow_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut SnowParticle, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut snow, mut transform) in particles.iter_mut() {
        snow.time_alive += dt;

        if snow.time_alive >= snow.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Move with velocity (swirl + drift down)
        transform.translation += snow.velocity * dt;

        // Lateral sway
        let t = snow.time_alive;
        let sway = (t * SNOW_SWAY_FREQUENCY * std::f32::consts::TAU + snow.phase).sin()
            * SNOW_SWAY_AMPLITUDE
            * dt;
        transform.translation.x += sway;

        // Fade in/out via scale
        let life_frac = snow.time_alive / snow.lifetime;
        let alpha = if life_frac < 0.15 {
            // Fade in
            life_frac / 0.15
        } else if life_frac > 0.75 {
            // Fade out
            (1.0 - life_frac) / 0.25
        } else {
            1.0
        };
        transform.scale = Vec3::splat(snow.base_size * alpha);
    }
}
