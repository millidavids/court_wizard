//! Fire glow, smoke wisps, and sparks visual effects.

use bevy::prelude::*;

use super::components::{FireGlow, FireSmoke, FireSpark};
use super::constants;
use super::constants::UPWARD_ROTATION;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub fn update_fire_glow(
    mut glow_query: Query<(&FireGlow, &mut Transform)>,
    source_query: Query<&Transform, Without<FireGlow>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(source_transform) = source_query.get(glow.source_entity) else {
            continue;
        };

        transform.translation = source_transform.translation;

        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let size = glow.base_radius * constants::GLOW_SIZE_MULTIPLIER * pulse;
        transform.scale = Vec3::splat(size);
    }
}

/// Despawns orphaned glow entities whose source has been despawned.
pub fn cleanup_orphaned_glows(
    mut commands: Commands,
    glow_query: Query<(Entity, &FireGlow)>,
    source_query: Query<Entity>,
) {
    for (glow_entity, glow) in glow_query.iter() {
        if source_query.get(glow.source_entity).is_err() {
            commands.entity(glow_entity).try_despawn();
        }
    }
}

/// Drifts, scale-fades, and despawns fire smoke wisps.
pub fn update_fire_smoke(
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut FireSmoke, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut smoke, mut transform) in smoke_query.iter_mut() {
        smoke.time_alive += dt;

        if smoke.time_alive >= smoke.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Drift by velocity
        transform.translation += smoke.velocity * dt;

        // Grow then shrink: peak at 60% of lifetime, shrink to zero by end
        let progress = smoke.time_alive / smoke.lifetime;
        let size = if progress < 0.6 {
            // Grow phase: 1.0 -> 1.5x over first 60%
            smoke.base_size * (1.0 + progress * 0.83)
        } else {
            // Shrink phase: 1.5x -> 0.0 over last 40%
            let shrink = 1.0 - (progress - 0.6) / 0.4;
            smoke.base_size * 1.5 * shrink
        };
        transform.scale = Vec3::splat(size);
    }
}

/// Moves, shrinks, and despawns spark particles (fire explosions and cast flares).
pub fn update_fire_sparks(
    mut commands: Commands,
    mut spark_query: Query<(Entity, &mut FireSpark, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut spark, mut transform) in spark_query.iter_mut() {
        spark.time_alive += dt;

        if spark.time_alive >= spark.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        transform.translation += spark.velocity * dt;
        transform.translation.y -= spark.gravity * dt * spark.time_alive;

        let remaining = 1.0 - (spark.time_alive / spark.lifetime);
        transform.scale = Vec3::splat(spark.base_size * remaining);
    }
}

/// Spawns a glow halo sibling for a fire projectile.
pub fn spawn_fire_glow(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    source_entity: Entity,
    position: Vec3,
    base_radius: f32,
) {
    commands.spawn((
        FireGlow {
            source_entity,
            base_radius,
        },
        Mesh3d(assets.particle_quad.clone()),
        MeshMaterial3d(assets.fire_glow.clone()),
        Transform::from_translation(position).with_rotation(UPWARD_ROTATION),
        OnGameplayScreen,
    ));
}

/// Spawns smoke wisps at a given position with upward drift.
#[allow(clippy::too_many_arguments)]
pub fn spawn_fire_smoke_wisps(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
    lifetime: f32,
    base_size: f32,
    rise_speed: f32,
    spread_speed: f32,
) {
    spawn_smoke_wisps_with_material(
        commands,
        assets,
        position,
        count,
        time_secs,
        lifetime,
        base_size,
        rise_speed,
        spread_speed,
        assets.fire_smoke.clone(),
    );
}

/// Spawns smoke wisps with a custom material handle.
#[allow(clippy::too_many_arguments)]
pub fn spawn_smoke_wisps_with_material(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
    lifetime: f32,
    base_size: f32,
    rise_speed: f32,
    spread_speed: f32,
    material: Handle<StandardMaterial>,
) {
    for i in 0..count {
        // Use multiple incommensurate frequencies + golden ratio to break uniform patterns
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5 + (seed * 31.3).cos() * 0.8;
        let spread_variation = 0.6 + 0.4 * ((seed * 17.3).sin() * 0.5 + 0.5);
        let rise_variation = 0.7 + 0.3 * ((seed * 23.1).cos() * 0.5 + 0.5);
        let lateral_x = angle.cos() * spread_speed * spread_variation;
        let lateral_z = angle.sin() * spread_speed * spread_variation;
        let velocity = Vec3::new(lateral_x, rise_speed * rise_variation, lateral_z);

        commands.spawn((
            FireSmoke {
                velocity,
                time_alive: 0.0,
                lifetime,
                base_size,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(position)
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(base_size)),
            OnGameplayScreen,
        ));
    }
}

/// Spawns impact sparks radiating outward from an explosion point.
pub fn spawn_fire_sparks(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    spawn_sparks_with_material(
        commands,
        assets,
        position,
        count,
        time_secs,
        assets.fire_spark.clone(),
    );
}

/// Spawns impact sparks with a custom material (e.g., white dispel sparks).
pub fn spawn_sparks_with_material(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
    material: Handle<StandardMaterial>,
) {
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU + time_secs * 3.7;
        // Low elevation: mostly horizontal with slight upward bias
        let elevation_range = constants::SPARK_ELEVATION_MAX - constants::SPARK_ELEVATION_MIN;
        let elevation =
            constants::SPARK_ELEVATION_MIN + (i as f32 * 1.618).fract() * elevation_range;
        let horizontal = (1.0 - elevation * elevation).sqrt();

        let velocity = Vec3::new(
            horizontal * angle.cos() * constants::SPARK_SPEED,
            elevation * constants::SPARK_SPEED,
            horizontal * angle.sin() * constants::SPARK_SPEED,
        );

        commands.spawn((
            FireSpark {
                velocity,
                time_alive: 0.0,
                lifetime: constants::SPARK_LIFETIME,
                base_size: constants::SPARK_SIZE,
                gravity: 200.0,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(position)
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(constants::SPARK_SIZE)),
            OnGameplayScreen,
        ));
    }
}
