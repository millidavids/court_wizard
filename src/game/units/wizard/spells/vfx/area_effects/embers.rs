use bevy::prelude::*;

use super::super::components::FireEmber;
use super::super::constants;
use crate::game::components::Billboard;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns flickering ember particles at the base of a fire effect.
pub fn spawn_fire_embers(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    half_width: f32,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 11.3;
        let angle = seed * 2.39 + (seed * 13.7).sin();
        let r = half_width * 0.4 * ((seed * 23.1).sin() * 0.5 + 0.5);
        let x = position.x + angle.cos() * r;
        let z = position.z + angle.sin() * r;

        let velocity = Vec3::new(
            angle.sin() * constants::EMBER_SPREAD_SPEED * ((seed * 17.3).cos() * 0.5 + 0.5),
            constants::EMBER_RISE_SPEED * (0.5 + 0.5 * (seed * 31.3).sin().abs()),
            -angle.cos() * constants::EMBER_SPREAD_SPEED * ((seed * 19.1).sin() * 0.5 + 0.5),
        );

        commands.spawn((
            FireEmber {
                velocity,
                time_alive: 0.0,
                lifetime: constants::EMBER_LIFETIME * (0.7 + 0.6 * (seed * 41.7).cos().abs()),
                base_size: constants::EMBER_SIZE * (0.5 + (seed * 53.3).sin().abs()),
                phase: seed,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(assets.fire_ember.clone()),
            Transform::from_translation(Vec3::new(x, position.y, z))
                .with_scale(Vec3::splat(constants::EMBER_SIZE)),
            Billboard,
            OnGameplayScreen,
        ));
    }
}

/// Updates fire embers: drift, flicker, fade, and despawn.
pub fn update_fire_embers(
    mut commands: Commands,
    mut ember_query: Query<(Entity, &mut FireEmber, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut ember, mut transform) in ember_query.iter_mut() {
        ember.time_alive += dt;
        if ember.time_alive >= ember.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        transform.translation += ember.velocity * dt;

        // Flicker via rapid scale oscillation
        let flicker = 0.5
            + 0.5
                * (t * constants::EMBER_FLICKER_FREQUENCY * std::f32::consts::TAU + ember.phase)
                    .sin();
        let remaining = 1.0 - (ember.time_alive / ember.lifetime);
        transform.scale = Vec3::splat(ember.base_size * remaining * (0.3 + 0.7 * flicker));
    }
}
