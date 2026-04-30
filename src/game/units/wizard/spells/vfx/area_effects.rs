//! Heat shimmer, plague/fog smoke, fire variants, and embers.

use bevy::prelude::*;

use super::components::{FireEmber, FireOrangeSmokePuff, HeatShimmer, PlagueSmoke};
use super::constants;
use super::constants::UPWARD_ROTATION;
use crate::game::components::Billboard;
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
    spawn_smoke_puffs(
        commands,
        assets,
        &assets.plague_smoke,
        &PLAGUE_SMOKE_PARAMS,
        center,
        cloud_radius,
        count,
        time_secs,
    );
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
    spawn_smoke_puffs(
        commands,
        assets,
        &assets.fog_smoke,
        &FOG_SMOKE_PARAMS,
        center,
        cloud_radius,
        count,
        time_secs,
    );
}

/// Spawns smoke puffs with configurable material and parameters.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_smoke_puffs(
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
        let velocity = Vec3::new(swirl_x, params.rise_speed * rise_variation, swirl_z);

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
                spawn_y: 0.0,
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
pub(super) fn spawn_fire_smoke_puff(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: Handle<StandardMaterial>,
    position: Vec3,
    velocity: Vec3,
    base_size: f32,
    lifetime: f32,
    seed: f32,
    extra: Option<FireOrangeSmokePuff>,
) {
    // Stagger initial time so batch-spawned particles don't pulse in sync
    let time_offset = ((seed * 3.7).fract().abs()) * lifetime * 0.25;
    let mut entity = commands.spawn((
        PlagueSmoke {
            velocity,
            time_alive: time_offset,
            lifetime,
            base_size,
            phase: seed,
            spawn_y: position.y,
        },
        Mesh3d(assets.particle_quad.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(Vec3::splat(base_size * 0.3)),
        Billboard,
        OnGameplayScreen,
    ));
    if let Some(marker) = extra {
        entity.insert(marker);
    }
}

/// Spawns a fire puff using the procedural `FireParticleMaterial` shader.
/// All particles share one material handle for GPU batching.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_fire_particle_puff(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    velocity: Vec3,
    base_size: f32,
    lifetime: f32,
    seed: f32,
    extra: Option<FireOrangeSmokePuff>,
) {
    let time_offset = ((seed * 3.7).fract().abs()) * lifetime * 0.25;
    let mut entity = commands.spawn((
        PlagueSmoke {
            velocity,
            time_alive: time_offset,
            lifetime,
            base_size,
            phase: seed,
            spawn_y: position.y,
        },
        Mesh3d(assets.particle_quad.clone()),
        MeshMaterial3d(assets.fire_particle.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(base_size * 0.3)),
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

        spawn_fire_particle_puff(
            commands,
            assets,
            Vec3::new(position.x, position.y + 8.0, position.z),
            velocity,
            base_size,
            1.5 * lifetime_variation,
            seed,
            None,
        );
    }
}

/// Spawns large, slow-rising dark smoke puffs using the plague-wind billboard style.
/// These are the visible plume smoke that rises above the fire — same animation system
/// as plague wind (`PlagueSmoke` component) but with dark gray material.
pub fn spawn_fire_rising_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    half_width: f32,
    count: usize,
    time_secs: f32,
) {
    for i in 0..count {
        let seed = i as f32 * 1.618_034 + time_secs * 5.3;
        let angle = seed * 2.39 + (seed * 13.7).sin();

        // Scatter within the fire area
        let r = half_width * 0.5 * ((seed * 23.1).sin() * 0.5 + 0.5);
        let x = position.x + angle.cos() * r;
        let z = position.z + angle.sin() * r;

        // Slow upward drift with gentle swirl (like plague wind)
        let rise_variation = 0.7 + 0.3 * ((seed * 17.3).cos() * 0.5 + 0.5);
        let velocity = Vec3::new(
            angle.sin() * constants::RISING_SMOKE_SWIRL_SPEED,
            constants::RISING_SMOKE_RISE_SPEED * rise_variation,
            -angle.cos() * constants::RISING_SMOKE_SWIRL_SPEED,
        );

        let size_variation = 0.6 + 0.4 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = constants::RISING_SMOKE_SIZE * size_variation;
        let lifetime_variation = 0.8 + 0.4 * ((seed * 53.3).cos() * 0.5 + 0.5);
        let time_offset = ((seed * 3.7).fract().abs()) * 0.3;

        commands.spawn((
            PlagueSmoke {
                velocity,
                time_alive: time_offset,
                lifetime: constants::RISING_SMOKE_LIFETIME * lifetime_variation,
                base_size,
                phase: seed,
                spawn_y: 0.0,
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(assets.smoke_particle.clone()),
            Transform::from_translation(Vec3::new(
                x,
                position.y + constants::RISING_SMOKE_Y_OFFSET,
                z,
            ))
            .with_scale(Vec3::splat(base_size * 0.3)),
            Billboard,
            OnGameplayScreen,
        ));
    }
}

/// Spawns dense fire particle clouds using the procedural `FireParticleMaterial` shader.
///
/// All particles share one material handle for GPU batching. The shader handles color
/// variation (noise → fire gradient), so no layer-based material selection is needed.
/// Three layers control size, speed, and lifetime:
/// - **Base (layer 0)**: small, slow rise, brief (fire flicker)
/// - **Mid (layer 1)**: medium, moderate rise
/// - **Top (layer 2)**: larger, fast rise, long-lived (rising smoke)
///
/// Also spawns flickering embers at the fire base.
pub fn spawn_fire_orange_smoke(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    half_width: f32,
    count: usize,
    time_secs: f32,
) {
    let actual_count = count * constants::FIRE_PARTICLE_COUNT_MULTIPLIER;

    for i in 0..actual_count {
        let seed = i as f32 * 1.618_034 + time_secs * 7.1;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5 + (seed * 31.3).cos() * 0.8;
        let layer = i % 3;

        let (rise_base, spread_mult, size_base) = match layer {
            0 => constants::FIRE_LAYER_BASE,
            1 => constants::FIRE_LAYER_MID,
            _ => constants::FIRE_LAYER_TOP,
        };

        let lateral_frac = (seed * 23.1).sin();
        let x = position.x + angle.cos() * half_width * lateral_frac * 0.6 * spread_mult;
        let z = position.z + angle.sin() * half_width * lateral_frac * 0.6 * spread_mult;

        let rise_variation = 0.6 + 0.4 * ((seed * 17.3).cos() * 0.5 + 0.5);
        let lateral_speed = constants::FIRE_LATERAL_SPEED * spread_mult;
        let velocity = Vec3::new(
            angle.sin() * lateral_speed,
            rise_base * rise_variation,
            -angle.cos() * lateral_speed,
        );

        let size_variation = 0.5 + 1.0 * ((seed * 41.7).sin() * 0.5 + 0.5);
        let base_size = size_base * size_variation;
        let lifetime_variation = 0.7 + 0.3 * ((seed * 53.3).cos() * 0.5 + 0.5);

        let base_lifetime = match layer {
            0 => constants::FIRE_LAYER_LIFETIME_BASE,
            1 => constants::FIRE_LAYER_LIFETIME_MID,
            _ => constants::FIRE_LAYER_LIFETIME_TOP,
        };

        // Base and mid layers emit apex black puffs (top layer already shows smoke via shader)
        let apex_marker = if layer < 2 {
            Some(FireOrangeSmokePuff { emitted: false })
        } else {
            None
        };

        spawn_fire_particle_puff(
            commands,
            assets,
            Vec3::new(x, position.y, z),
            velocity,
            base_size,
            base_lifetime * lifetime_variation,
            seed,
            apex_marker,
        );
    }

    // Spawn flickering embers at the base
    let ember_count = (count * 2).max(2);
    spawn_fire_embers(
        commands,
        assets,
        position,
        half_width,
        ember_count,
        time_secs,
    );

    // Spawn rising dark smoke above the fire (plague-wind style billowing)
    let smoke_count = count.max(1);
    spawn_fire_rising_smoke(
        commands,
        assets,
        position,
        half_width,
        smoke_count,
        time_secs,
    );
}

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
            spawn_fire_black_smoke(&mut commands, &assets, transform.translation, 1, t);
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

        // Height-based effects for fire puffs (spawn_y > 0 means this is a fire puff)
        let is_fire = smoke.spawn_y > 0.0;
        let height_factor = if is_fire {
            let height_risen = (transform.translation.y - smoke.spawn_y).max(0.0);
            (height_risen / constants::FIRE_HEIGHT_SCALE_RANGE).clamp(0.0, 2.0)
        } else {
            0.0
        };

        // Lateral sway — fire puffs widen as they rise
        let sway_mult = if is_fire {
            1.0 + height_factor * constants::FIRE_HEIGHT_SWAY_MULT
        } else {
            1.0
        };
        let sway_amp = constants::PLAGUE_SMOKE_SWAY_AMPLITUDE * sway_mult;
        let sway_x = (t * constants::PLAGUE_SMOKE_SWAY_FREQUENCY * std::f32::consts::TAU
            + smoke.phase)
            .sin()
            * sway_amp
            * dt;
        transform.translation.x += sway_x;

        // Fire puffs also get Z-axis sway for 3D billowing
        if is_fire {
            let sway_z = (t * constants::PLAGUE_SMOKE_SWAY_FREQUENCY * 0.7 * std::f32::consts::TAU
                + smoke.phase
                + 1.5)
                .cos()
                * sway_amp
                * 0.6
                * dt;
            transform.translation.z += sway_z;
        }

        // Scale: grow quickly in first 30%, hold, then shrink in last 30%
        let base_scale = if progress < 0.3 {
            let grow = progress / 0.3;
            smoke.base_size * (0.3 + 0.7 * grow)
        } else if progress > 0.7 {
            let shrink = 1.0 - (progress - 0.7) / 0.3;
            smoke.base_size * shrink
        } else {
            smoke.base_size
        };

        // Fire puffs grow larger as they rise
        let height_scale = if is_fire {
            1.0 + height_factor * constants::FIRE_HEIGHT_SIZE_MULT
        } else {
            1.0
        };
        transform.scale = Vec3::splat(base_scale * height_scale);
    }
}
