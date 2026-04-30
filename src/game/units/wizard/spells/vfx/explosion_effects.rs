//! Explosion smoke and missile visual effects.

use bevy::prelude::*;

use super::area_effects::spawn_fire_smoke_puff;
use super::components::{MissileGlow, MissileSparkle};
use super::constants;
use super::constants::UPWARD_ROTATION;
use super::fire_effects::spawn_smoke_wisps_with_material;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub fn spawn_explosion_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    time_secs: f32,
) {
    spawn_explosion_smoke_with_material(
        commands,
        assets,
        position,
        time_secs,
        assets.fire_smoke.clone(),
        constants::EXPLOSION_SMOKE_COUNT,
    );
}

/// Spawns explosion smoke with a custom material and particle count.
pub fn spawn_explosion_smoke_with_material(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    time_secs: f32,
    material: Handle<StandardMaterial>,
    count: usize,
) {
    spawn_smoke_wisps_with_material(
        commands,
        assets,
        position,
        count,
        time_secs,
        constants::EXPLOSION_SMOKE_LIFETIME,
        constants::EXPLOSION_SMOKE_SIZE,
        constants::EXPLOSION_SMOKE_RISE_SPEED,
        constants::EXPLOSION_SMOKE_SPREAD,
        material,
    );
}

/// Spawns lingering dark smoke puffs that drift slowly upward after an explosion.
pub fn spawn_explosion_dark_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    time_secs: f32,
) {
    for i in 0..constants::DARK_SMOKE_COUNT {
        let seed = i as f32 * 1.618_034 + time_secs * 5.3;
        let angle = seed * 2.39 + (seed * 11.7).sin() * 1.2;
        let spread_variation = 0.5 + 0.5 * ((seed * 19.1).sin() * 0.5 + 0.5);
        let rise_variation = 0.6 + 0.4 * ((seed * 27.3).cos() * 0.5 + 0.5);
        let lateral_x = angle.cos() * constants::DARK_SMOKE_SPREAD_SPEED * spread_variation;
        let lateral_z = angle.sin() * constants::DARK_SMOKE_SPREAD_SPEED * spread_variation;
        let velocity = Vec3::new(
            lateral_x,
            constants::DARK_SMOKE_RISE_SPEED * rise_variation,
            lateral_z,
        );
        let phase = seed * std::f32::consts::PI + (seed * 37.1).sin();

        spawn_fire_smoke_puff(
            commands,
            assets,
            assets.fire_black_smoke.clone(),
            position,
            velocity,
            constants::DARK_SMOKE_SIZE,
            constants::DARK_SMOKE_LIFETIME,
            phase,
            None,
        );
    }
}

// ── Magic missile VFX ──────────────────────────────────────────────

/// Spawns a glow halo sibling for a magic missile.
pub fn spawn_missile_glow(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    source_entity: Entity,
    position: Vec3,
    base_radius: f32,
) {
    commands.spawn((
        MissileGlow {
            source_entity,
            base_radius,
        },
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.missile_glow.clone()),
        Transform::from_translation(position),
        OnGameplayScreen,
    ));
}

/// Updates missile glow position and pulsing scale to follow its source entity.
pub fn update_missile_glow(
    mut glow_query: Query<(&MissileGlow, &mut Transform)>,
    source_query: Query<&Transform, Without<MissileGlow>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(source_transform) = source_query.get(glow.source_entity) else {
            continue;
        };

        transform.translation = source_transform.translation;

        let pulse = 1.0
            + constants::MISSILE_GLOW_PULSE_AMPLITUDE
                * (t * constants::MISSILE_GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let size = glow.base_radius * constants::MISSILE_GLOW_SIZE_MULTIPLIER * pulse;
        transform.scale = Vec3::splat(size);
    }
}

/// Despawns orphaned missile glow entities whose source has been despawned.
pub fn cleanup_orphaned_missile_glows(
    mut commands: Commands,
    glow_query: Query<(Entity, &MissileGlow)>,
    source_query: Query<Entity>,
) {
    for (glow_entity, glow) in glow_query.iter() {
        if source_query.get(glow.source_entity).is_err() {
            commands.entity(glow_entity).try_despawn();
        }
    }
}

/// Spawns sparkle particles at a missile's position that inherit its velocity
/// then decelerate, creating a comet-like trail.
pub fn spawn_missile_sparkles(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    missile_velocity: Vec3,
    time_secs: f32,
) {
    for i in 0..constants::SPARKLE_COUNT_PER_SPAWN {
        // Inherit missile velocity (scaled down) with some random spread
        let seed = i as f32 * 1.618_034 + time_secs * 11.3;
        let spread_x = (seed * 7.3).sin() * constants::SPARKLE_SPREAD_SPEED;
        let spread_y = (seed * 13.1).cos() * constants::SPARKLE_SPREAD_SPEED * 0.5;
        let spread_z = (seed * 19.7).sin() * constants::SPARKLE_SPREAD_SPEED;

        // Start with a fraction of the missile's velocity so they lag behind
        let velocity = missile_velocity * 0.3 + Vec3::new(spread_x, spread_y, spread_z);

        commands.spawn((
            MissileSparkle {
                velocity,
                time_alive: 0.0,
                lifetime: constants::SPARKLE_LIFETIME,
                base_size: constants::SPARKLE_SIZE,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(assets.missile_sparkle.clone()),
            Transform::from_translation(position)
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(constants::SPARKLE_SIZE)),
            OnGameplayScreen,
        ));
    }
}

/// Decelerates, shrinks, and despawns missile sparkle particles.
pub fn update_missile_sparkles(
    mut commands: Commands,
    mut sparkle_query: Query<(Entity, &mut MissileSparkle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut sparkle, mut transform) in sparkle_query.iter_mut() {
        sparkle.time_alive += dt;

        if sparkle.time_alive >= sparkle.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Decelerate (exponential drag)
        let drag = (-constants::SPARKLE_DRAG * dt).exp();
        sparkle.velocity *= drag;

        // Move by velocity
        transform.translation += sparkle.velocity * dt;

        // Shrink linearly over lifetime
        let remaining = 1.0 - (sparkle.time_alive / sparkle.lifetime);
        let size = sparkle.base_size * remaining;
        transform.scale = Vec3::splat(size);
    }
}
