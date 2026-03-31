//! Shared visual effect systems.

use bevy::prelude::*;

use super::components::{
    CastFlare, FireGlow, FireOrangeSmokePuff, FireSmoke, FireSpark, FloatingMote,
    HeatShimmer, MissileGlow, MissileSparkle, PlagueSmoke, SmokePoof,
};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::components::Billboard;
use super::constants;
use super::constants::UPWARD_ROTATION;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Updates glow halo position and pulsing scale to follow its source entity.
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
            MeshMaterial3d(assets.fire_smoke.clone()),
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

/// Spawns a burst of smoke from an explosion point.
pub fn spawn_explosion_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    time_secs: f32,
) {
    spawn_fire_smoke_wisps(
        commands,
        assets,
        position,
        constants::EXPLOSION_SMOKE_COUNT,
        time_secs,
        constants::EXPLOSION_SMOKE_LIFETIME,
        constants::EXPLOSION_SMOKE_SIZE,
        constants::EXPLOSION_SMOKE_RISE_SPEED,
        constants::EXPLOSION_SMOKE_SPREAD,
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
            &assets.particle_quad,
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

// ── Heat shimmer VFX ──────────────────────────────────────────────

/// Spawns heat shimmer billboards at a given position for a lo-fi heat haze.
pub fn spawn_heat_shimmer(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    spawn_heat_shimmer_sized(commands, assets, position, count, time_secs, constants::SHIMMER_SIZE);
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

// ── Plague smoke (poison cloud particles) ─────────────────────────────

/// Spawns plague smoke puffs scattered within a cloud volume.
/// Each puff is a billboard that drifts upward with gentle swirling.
/// Tunable parameters for smoke puff spawning.
pub struct SmokePuffParams {
    pub rise_speed: f32,
    pub swirl_speed: f32,
    pub size: f32,
    pub lifetime: f32,
    /// Height base multiplier (fraction of rise_speed for minimum height).
    pub height_base: f32,
    /// Height range multiplier (fraction of rise_speed added by randomness).
    pub height_range: f32,
}

/// Plague wind smoke parameters (standard cloud).
pub const PLAGUE_SMOKE_PARAMS: SmokePuffParams = SmokePuffParams {
    rise_speed: constants::PLAGUE_SMOKE_RISE_SPEED,
    swirl_speed: constants::PLAGUE_SMOKE_SWIRL_SPEED,
    size: constants::PLAGUE_SMOKE_SIZE,
    lifetime: constants::PLAGUE_SMOKE_LIFETIME,
    height_base: 0.3,
    height_range: 0.5,
};

/// Fog cloud smoke parameters (denser, ground-hugging).
pub const FOG_SMOKE_PARAMS: SmokePuffParams = SmokePuffParams {
    rise_speed: constants::FOG_SMOKE_RISE_SPEED,
    swirl_speed: constants::FOG_SMOKE_SWIRL_SPEED,
    size: constants::FOG_SMOKE_SIZE,
    lifetime: constants::FOG_SMOKE_LIFETIME,
    height_base: 0.2,
    height_range: 0.3,
};

/// Spawns plague smoke puffs (green).
pub fn spawn_plague_smoke_puffs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    center: Vec3,
    cloud_radius: f32,
    count: usize,
    time_secs: f32,
) {
    spawn_smoke_puffs(commands, assets, &assets.plague_smoke, &PLAGUE_SMOKE_PARAMS, center, cloud_radius, count, time_secs);
}

/// Spawns fog smoke puffs (gray, denser and ground-hugging).
pub fn spawn_fog_smoke_puffs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    center: Vec3,
    cloud_radius: f32,
    count: usize,
    time_secs: f32,
) {
    spawn_smoke_puffs(commands, assets, &assets.fog_smoke, &FOG_SMOKE_PARAMS, center, cloud_radius, count, time_secs);
}

/// Spawns smoke puffs with configurable material and parameters.
#[allow(clippy::too_many_arguments)]
fn spawn_smoke_puffs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: &Handle<StandardMaterial>,
    params: &SmokePuffParams,
    center: Vec3,
    cloud_radius: f32,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5;

        // Scatter within cloud radius (weighted toward edges for more volume)
        let r_frac = 0.2 + 0.8 * ((seed * 23.1).sin() * 0.5 + 0.5);
        let r = cloud_radius * r_frac * 0.7;
        let x = center.x + angle.cos() * r;
        let z = center.z + angle.sin() * r;

        // Random height within cloud volume
        let height_frac = (seed * 31.3).cos() * 0.5 + 0.5;
        let y = params.rise_speed * params.height_base
            + height_frac * params.rise_speed * params.height_range;

        // Gentle upward drift with swirl
        let rise_variation = 0.7 + 0.3 * ((seed * 17.3).cos() * 0.5 + 0.5);
        let swirl_x = angle.sin() * params.swirl_speed;
        let swirl_z = -angle.cos() * params.swirl_speed;
        let velocity = Vec3::new(
            swirl_x,
            params.rise_speed * rise_variation,
            swirl_z,
        );

        let size_variation = 0.6 + 0.4 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = params.size * size_variation;

        let lifetime_variation = 0.8 + 0.4 * ((seed * 53.3).cos() * 0.5 + 0.5);

        commands.spawn((
            PlagueSmoke {
                velocity,
                time_alive: 0.0,
                lifetime: params.lifetime * lifetime_variation,
                base_size,
                phase: seed,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(x, y, z))
                .with_scale(Vec3::splat(base_size * 0.3)),
            Billboard,
            OnGameplayScreen,
        ));
    }
}

/// Shared helper for spawning fire smoke billboard puffs (black or orange).
///
/// Each puff reuses `PlagueSmoke` for its drift/sway/scale lifecycle.
#[allow(clippy::too_many_arguments)]
fn spawn_fire_smoke_puff(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    velocity: Vec3,
    base_size: f32,
    lifetime: f32,
    seed: f32,
    extra: Option<FireOrangeSmokePuff>,
) {
    let mut entity = commands.spawn((
        PlagueSmoke {
            velocity,
            time_alive: 0.0,
            lifetime,
            base_size,
            phase: seed,
        },
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(position)
            .with_scale(Vec3::splat(base_size * 0.3)),
        Billboard,
        OnGameplayScreen,
    ));
    if let Some(marker) = extra {
        entity.insert(marker);
    }
}

/// Spawns black smoke puffs rising from a fire source.
pub fn spawn_fire_black_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5 + (seed * 31.3).cos() * 0.8;

        let rise_variation = 0.7 + 0.3 * ((seed * 17.3).cos() * 0.5 + 0.5);
        let velocity = Vec3::new(angle.sin() * 6.0, 20.0 * rise_variation, -angle.cos() * 6.0);

        let size_variation = 0.6 + 0.4 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = 18.0 * size_variation;
        let lifetime_variation = 0.8 + 0.4 * ((seed * 53.3).cos() * 0.5 + 0.5);

        spawn_fire_smoke_puff(
            commands,
            &assets.particle_quad,
            assets.fire_black_smoke.clone(),
            Vec3::new(position.x, position.y + 8.0, position.z),
            velocity,
            base_size,
            1.5 * lifetime_variation,
            seed,
            None,
        );
    }
}

/// Spawns thick orange fire smoke puffs near the ground.
/// Scattered within the wall's half-width, slow rise, large particles.
/// Each puff will emit a black smoke puff at its apex via [`emit_fire_smoke_apex_puffs`].
pub fn spawn_fire_orange_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    half_width: f32,
    count: usize,
    time_secs: f32,
) {
    // Material variants for natural color variation
    let materials = [
        &assets.fire_orange_smoke,
        &assets.fire_orange_smoke_light,
        &assets.fire_orange_smoke_deep,
    ];

    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5 + (seed * 31.3).cos() * 0.8;

        // Scatter within the wall width
        let lateral_frac = (seed * 23.1).sin();
        let x = position.x + angle.cos() * half_width * lateral_frac * 0.6;
        let z = position.z + angle.sin() * half_width * lateral_frac * 0.6;

        let rise_variation = 0.6 + 0.4 * ((seed * 17.3).cos() * 0.5 + 0.5);
        let velocity = Vec3::new(angle.sin() * 4.0, 8.0 * rise_variation, -angle.cos() * 4.0);

        // Wider size range for natural variation (10–30 base units)
        let size_variation = 0.5 + 1.0 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = 20.0 * size_variation;
        let lifetime_variation = 0.8 + 0.4 * ((seed * 53.3).cos() * 0.5 + 0.5);

        let mat_index = ((seed * 7.7).abs() as usize) % materials.len();

        spawn_fire_smoke_puff(
            commands,
            &assets.particle_quad,
            materials[mat_index].clone(),
            Vec3::new(x, 2.0, z),
            velocity,
            base_size,
            1.2 * lifetime_variation,
            seed,
            Some(FireOrangeSmokePuff { emitted: false }),
        );
    }
}

/// Emits a single black smoke puff from each orange fire smoke puff at its apex (30% lifetime).
pub fn emit_fire_smoke_apex_puffs(
    mut commands: Commands,
    mut puffs: Query<(&PlagueSmoke, &Transform, &mut FireOrangeSmokePuff)>,
    assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (smoke, transform, mut marker) in puffs.iter_mut() {
        if marker.emitted {
            continue;
        }
        let progress = smoke.time_alive / smoke.lifetime;
        if progress >= 0.3 {
            marker.emitted = true;
            spawn_fire_black_smoke(
                &mut commands,
                &assets,
                transform.translation,
                1,
                t,
            );
        }
    }
}

/// Updates plague smoke puffs: drift, sway, grow, then fade.
pub fn update_plague_smoke(
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut PlagueSmoke, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut smoke, mut transform) in smoke_query.iter_mut() {
        smoke.time_alive += dt;

        if smoke.time_alive >= smoke.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = smoke.time_alive / smoke.lifetime;

        // Drift upward + swirl
        transform.translation += smoke.velocity * dt;

        // Add gentle lateral sway for organic movement
        let sway = (t * constants::PLAGUE_SMOKE_SWAY_FREQUENCY * std::f32::consts::TAU
            + smoke.phase)
            .sin()
            * constants::PLAGUE_SMOKE_SWAY_AMPLITUDE
            * dt;
        transform.translation.x += sway;

        // Scale: grow quickly in first 30%, hold, then shrink in last 30%
        let size = if progress < 0.3 {
            let grow = progress / 0.3;
            smoke.base_size * (0.3 + 0.7 * grow)
        } else if progress > 0.7 {
            let shrink = 1.0 - (progress - 0.7) / 0.3;
            smoke.base_size * shrink
        } else {
            smoke.base_size
        };
        transform.scale = Vec3::splat(size);
    }
}

// ── Cast flare VFX (muzzle flash at wizard staff) ────────────────────

/// Spawns a cast flare (expanding glow + colored sparks) at the given position.
/// Call this from each spell's casting system when the spell completes.
pub fn spawn_cast_flare(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    glow_material: Handle<StandardMaterial>,
    spark_material: Handle<StandardMaterial>,
    time_secs: f32,
) {
    // Expanding glow ring
    commands.spawn((
        CastFlare {
            time_alive: 0.0,
            lifetime: constants::CAST_FLARE_LIFETIME,
            base_size: constants::CAST_FLARE_SIZE,
        },
        Mesh3d(assets.particle_quad.clone()),
        MeshMaterial3d(glow_material),
        Transform::from_translation(position)
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(constants::CAST_FLARE_SIZE * 0.3)),
        OnGameplayScreen,
    ));

    // Colored sparks radiating outward
    for i in 0..constants::CAST_FLARE_SPARK_COUNT {
        let angle = (i as f32 / constants::CAST_FLARE_SPARK_COUNT as f32) * std::f32::consts::TAU
            + time_secs * 5.3;
        let elevation = 0.1 + (i as f32 * 1.618).fract() * 0.4;
        let horizontal = (1.0 - elevation * elevation).sqrt();

        let velocity = Vec3::new(
            horizontal * angle.cos() * constants::CAST_FLARE_SPARK_SPEED,
            elevation * constants::CAST_FLARE_SPARK_SPEED,
            horizontal * angle.sin() * constants::CAST_FLARE_SPARK_SPEED,
        );

        commands.spawn((
            FireSpark {
                velocity,
                time_alive: 0.0,
                lifetime: constants::CAST_FLARE_SPARK_LIFETIME,
                base_size: constants::CAST_FLARE_SPARK_SIZE,
                gravity: 150.0,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(spark_material.clone()),
            Transform::from_translation(position)
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(constants::CAST_FLARE_SPARK_SIZE)),
            OnGameplayScreen,
        ));
    }
}

/// Spell school categories for cast flare coloring.
#[derive(Clone, Copy)]
pub enum SpellSchool {
    Fire,
    Lightning,
    Arcane,
    Nature,
    Holy,
    Dark,
    Force,
    Transmutation,
}

/// Spawns a cast flare at SPELL_ORIGIN using the given spell school's colors.
/// Convenience wrapper around `spawn_cast_flare` for use in spell cast functions.
pub fn spawn_school_flare(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    school: SpellSchool,
    time_secs: f32,
) {
    let (glow, spark) = match school {
        SpellSchool::Fire => (assets.flare_fire_glow.clone(), assets.flare_fire_spark.clone()),
        SpellSchool::Lightning => (assets.flare_lightning_glow.clone(), assets.flare_lightning_spark.clone()),
        SpellSchool::Arcane => (assets.flare_arcane_glow.clone(), assets.flare_arcane_spark.clone()),
        SpellSchool::Nature => (assets.flare_nature_glow.clone(), assets.flare_nature_spark.clone()),
        SpellSchool::Holy => (assets.flare_holy_glow.clone(), assets.flare_holy_spark.clone()),
        SpellSchool::Dark => (assets.flare_dark_glow.clone(), assets.flare_dark_spark.clone()),
        SpellSchool::Force => (assets.flare_force_glow.clone(), assets.flare_force_spark.clone()),
        SpellSchool::Transmutation => (assets.flare_transmutation_glow.clone(), assets.flare_transmutation_spark.clone()),
    };
    spawn_cast_flare(commands, assets, SPELL_ORIGIN, glow, spark, time_secs);
}

/// Updates cast flare glow: expands and fades over lifetime.
pub fn update_cast_flares(
    mut commands: Commands,
    mut flare_query: Query<(Entity, &mut CastFlare, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut flare, mut transform) in flare_query.iter_mut() {
        flare.time_alive += dt;

        if flare.time_alive >= flare.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = flare.time_alive / flare.lifetime;
        // Quick expand then shrink: peak at 30% of lifetime
        let scale = if progress < 0.3 {
            let grow = progress / 0.3;
            flare.base_size * (0.3 + 0.7 * grow)
        } else {
            let shrink = 1.0 - (progress - 0.3) / 0.7;
            flare.base_size * shrink
        };
        transform.scale = Vec3::splat(scale);
    }
}

// ── Floating mote VFX (ambient zone particles) ──────────────────────

/// Spawns floating mote particles scattered within a zone radius.
#[allow(clippy::too_many_arguments)]
pub fn spawn_floating_motes(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: &Handle<StandardMaterial>,
    center: Vec3,
    radius: f32,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 9.3;
        let angle = seed * 2.39 + (seed * 11.7).sin() * 1.3;

        // Scatter within zone radius
        let r_frac = 0.1 + 0.9 * ((seed * 19.1).sin() * 0.5 + 0.5);
        let r = radius * r_frac * 0.8;
        let x = center.x + angle.cos() * r;
        let z = center.z + angle.sin() * r;

        let rise_variation = 0.6 + 0.4 * ((seed * 23.1).cos() * 0.5 + 0.5);
        let lateral_x = (seed * 7.3).sin() * constants::MOTE_SPREAD_SPEED * 0.5;
        let lateral_z = (seed * 13.1).cos() * constants::MOTE_SPREAD_SPEED * 0.5;
        let velocity = Vec3::new(
            lateral_x,
            constants::MOTE_RISE_SPEED * rise_variation,
            lateral_z,
        );

        let size_variation = 0.5 + 0.5 * ((seed * 37.3).sin() * 0.5 + 0.5);
        let lifetime_variation = 0.7 + 0.6 * ((seed * 41.7).cos() * 0.5 + 0.5);

        commands.spawn((
            FloatingMote {
                velocity,
                time_alive: 0.0,
                lifetime: constants::MOTE_LIFETIME * lifetime_variation,
                base_size: constants::MOTE_SIZE * size_variation,
                phase: seed,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(x, center.y + 5.0, z))
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(0.0)),
            OnGameplayScreen,
        ));
    }
}

/// Updates floating motes: drift, sway, grow-then-fade, despawn.
pub fn update_floating_motes(
    mut commands: Commands,
    mut mote_query: Query<(Entity, &mut FloatingMote, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut mote, mut transform) in mote_query.iter_mut() {
        mote.time_alive += dt;

        if mote.time_alive >= mote.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = mote.time_alive / mote.lifetime;

        // Drift by velocity
        transform.translation += mote.velocity * dt;

        // Gentle lateral sway
        let sway = (t * constants::MOTE_SWAY_FREQUENCY * std::f32::consts::TAU + mote.phase)
            .sin()
            * constants::MOTE_SWAY_AMPLITUDE
            * dt;
        transform.translation.x += sway;

        // Fade in first 20%, hold, fade out last 30%
        let size = if progress < 0.2 {
            mote.base_size * (progress / 0.2)
        } else if progress > 0.7 {
            mote.base_size * (1.0 - (progress - 0.7) / 0.3)
        } else {
            mote.base_size
        };
        transform.scale = Vec3::splat(size);
    }
}

// ── Smoke poof VFX (transformation/banishment effects) ──────────────

/// Spawns a burst of smoke poof particles at a position.
pub fn spawn_smoke_poof(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: &Handle<StandardMaterial>,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.7;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5;

        let spread_variation = 0.5 + 0.5 * ((seed * 17.3).sin() * 0.5 + 0.5);
        let rise_variation = 0.6 + 0.4 * ((seed * 23.1).cos() * 0.5 + 0.5);
        let velocity = Vec3::new(
            angle.cos() * constants::POOF_SPREAD_SPEED * spread_variation,
            constants::POOF_RISE_SPEED * rise_variation,
            angle.sin() * constants::POOF_SPREAD_SPEED * spread_variation,
        );

        let size_variation = 0.6 + 0.8 * ((seed * 31.3).sin() * 0.5 + 0.5);

        commands.spawn((
            SmokePoof {
                velocity,
                time_alive: 0.0,
                lifetime: constants::POOF_LIFETIME,
                base_size: constants::POOF_SIZE * size_variation,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(position)
                .with_rotation(UPWARD_ROTATION)
                .with_scale(Vec3::splat(constants::POOF_SIZE * size_variation * 0.3)),
            Billboard,
            OnGameplayScreen,
        ));
    }
}

/// Updates smoke poof particles: expand quickly, drift, then fade.
pub fn update_smoke_poofs(
    mut commands: Commands,
    mut poof_query: Query<(Entity, &mut SmokePoof, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut poof, mut transform) in poof_query.iter_mut() {
        poof.time_alive += dt;

        if poof.time_alive >= poof.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = poof.time_alive / poof.lifetime;

        // Drift by velocity (decelerating)
        let drag = (-2.0 * dt).exp();
        poof.velocity *= drag;
        transform.translation += poof.velocity * dt;

        // Quick expand then shrink
        let size = if progress < 0.25 {
            poof.base_size * (0.3 + 2.8 * progress)
        } else {
            poof.base_size * (1.0 - (progress - 0.25) / 0.75)
        };
        transform.scale = Vec3::splat(size);
    }
}

/// Spawns earthy brown dust puffs for wall of stone rising/sinking effects.
/// Similar to orange fire smoke but with brown tones, slower rise, and more lateral spread.
pub fn spawn_dust_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    half_width: f32,
    count: usize,
    time_secs: f32,
) {
    let materials = [
        &assets.dust_smoke,
        &assets.dust_smoke_light,
        &assets.dust_smoke_dark,
    ];

    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5 + (seed * 31.3).cos() * 0.8;

        // Scatter beyond the wall width for a wider dust cloud
        let lateral_frac = (seed * 23.1).sin();
        let spread = half_width * 1.5;
        let x = position.x + angle.cos() * spread * lateral_frac;
        let z = position.z + angle.sin() * spread * lateral_frac;

        let rise_variation = 0.5 + 0.5 * ((seed * 17.3).cos() * 0.5 + 0.5);
        // Slower rise than fire, more lateral spread for dusty feel
        let velocity = Vec3::new(angle.sin() * 7.0, 6.0 * rise_variation, -angle.cos() * 7.0);

        let size_variation = 0.6 + 1.0 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = 16.0 * size_variation;
        let lifetime_variation = 0.8 + 0.4 * ((seed * 53.3).cos() * 0.5 + 0.5);

        let mat_index = ((seed * 7.7).abs() as usize) % materials.len();

        spawn_fire_smoke_puff(
            commands,
            &assets.particle_quad,
            materials[mat_index].clone(),
            Vec3::new(x, 2.0, z),
            velocity,
            base_size,
            1.0 * lifetime_variation,
            seed,
            None,
        );
    }
}
