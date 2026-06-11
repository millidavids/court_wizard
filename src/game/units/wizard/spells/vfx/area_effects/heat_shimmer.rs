use bevy::prelude::*;

use super::super::components::HeatShimmer;
use super::super::constants;
use super::super::constants::UPWARD_ROTATION;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub fn spawn_heat_shimmer(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    spawn_heat_shimmer_sized(
        commands,
        assets,
        position,
        count,
        time_secs,
        constants::SHIMMER_SIZE,
    );
}

/// Spawns heat shimmer billboards with a custom size (for larger surface fire effects).
pub fn spawn_heat_shimmer_sized(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
    base_size: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5 + (seed * 31.3).cos() * 0.8;
        let spread_variation = 0.6 + 0.4 * ((seed * 17.3).sin() * 0.5 + 0.5);
        let rise_variation = 0.7 + 0.3 * ((seed * 23.1).cos() * 0.5 + 0.5);
        let lateral_x = angle.cos() * constants::SHIMMER_SPREAD_SPEED * spread_variation;
        let lateral_z = angle.sin() * constants::SHIMMER_SPREAD_SPEED * spread_variation;
        let velocity = Vec3::new(
            lateral_x,
            constants::SHIMMER_RISE_SPEED * rise_variation,
            lateral_z,
        );

        let phase = seed * std::f32::consts::PI + (seed * 41.7).sin();

        commands.spawn((
            HeatShimmer {
                velocity,
                time_alive: 0.0,
                lifetime: constants::SHIMMER_LIFETIME,
                base_size,
                phase,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(assets.heat_shimmer.clone()),
            Transform::from_translation(position)
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(0.0)),
            OnGameplayScreen,
        ));
    }
}

/// Drifts, sways, scale-fades, and despawns heat shimmer particles.
pub fn update_heat_shimmer(
    mut commands: Commands,
    mut shimmer_query: Query<(Entity, &mut HeatShimmer, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut shimmer, mut transform) in shimmer_query.iter_mut() {
        shimmer.time_alive += dt;

        if shimmer.time_alive >= shimmer.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = shimmer.time_alive / shimmer.lifetime;

        // Drift upward by velocity
        transform.translation += shimmer.velocity * dt;

        // Lateral sway oscillation
        let sway = (t * constants::SHIMMER_SWAY_FREQUENCY * std::f32::consts::TAU + shimmer.phase)
            .sin()
            * constants::SHIMMER_SWAY_AMPLITUDE
            * dt;
        transform.translation.x += sway;

        // Scale: fade in over first 20%, stable, fade out over last 20%
        let scale_factor = if progress < 0.2 {
            progress / 0.2
        } else if progress > 0.8 {
            (1.0 - progress) / 0.2
        } else {
            1.0
        };
        transform.scale = Vec3::splat(shimmer.base_size * scale_factor);
    }
}
