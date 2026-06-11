use bevy::prelude::*;

use super::super::components::{FireOrangeSmokePuff, PlagueSmoke};
use super::super::constants;
use super::embers::spawn_fire_embers;
use crate::game::components::Billboard;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Shared helper for spawning fire smoke billboard puffs (black or orange).
///
/// Each puff reuses `PlagueSmoke` for its drift/sway/scale lifecycle.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_fire_smoke_puff(
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
pub(crate) fn spawn_fire_particle_puff(
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
