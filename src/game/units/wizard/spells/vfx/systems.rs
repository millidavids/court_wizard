//! Shared visual effect systems.

use bevy::prelude::*;

use super::components::{FireGlow, FireSmoke, FireSpark, HeatShimmer, MissileGlow, MissileSparkle};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Rotation to make a circle mesh (XY plane) lie flat facing upward (XZ plane).
const UPWARD_ROTATION: Quat = Quat::from_xyzw(
    -std::f32::consts::FRAC_1_SQRT_2,
    0.0,
    0.0,
    std::f32::consts::FRAC_1_SQRT_2,
);

/// Color palette for animated fire/effect visuals.
struct EffectPalette {
    /// Base RGB channels: (center, amplitude) for each of R, G, B
    r: (f32, f32),
    g_base: f32,
    g_amp1: f32,
    g_amp2: f32,
    b: (f32, f32),
    /// Emissive: (base_strength, green_factor, blue_mult)
    emissive_base: f32,
    emissive_green_factor: f32,
    emissive_blue_mult: f32,
}

const FIRE_PALETTE: EffectPalette = EffectPalette {
    r: (0.9, 0.1),
    g_base: 0.35,
    g_amp1: 0.2,
    g_amp2: 0.1,
    b: (0.0, 0.05), // special: uses sin*0.5+0.5 pattern
    emissive_base: 2.0,
    emissive_green_factor: 1.5,
    emissive_blue_mult: 0.5,
};

const POOP_PALETTE: EffectPalette = EffectPalette {
    r: (0.40, 0.05),
    g_base: 0.25,
    g_amp1: 0.05,
    g_amp2: 0.0,
    b: (0.08, 0.02),
    emissive_base: 0.8,
    emissive_green_factor: 0.4,
    emissive_blue_mult: 0.3,
};

/// Computes an organic, time-varying color with layered sine-wave cycling.
///
/// Returns `(base_color, emissive)` using the given palette for color ranges.
///
/// * `time` — elapsed seconds (drives the oscillation)
/// * `fade` — 0.0–1.0 multiplier for overall alpha (e.g. expiry fade-out)
fn animated_color_at(time: f32, fade: f32, p: &EffectPalette) -> (Color, LinearRgba) {
    let t = time;

    // Flicker envelope (3-layer sine)
    let flicker = 0.7 + 0.15 * (t * 8.3).sin() + 0.10 * (t * 13.7).sin() + 0.05 * (t * 23.1).sin();

    let r = p.r.0 + p.r.1 * (t * 5.3).sin();
    let g = p.g_base + p.g_amp1 * (t * 11.0).sin() + p.g_amp2 * (t * 7.3).sin();
    let b = (p.b.0 + p.b.1 * (t * 7.3).sin()).max(0.0);

    let alpha = 0.45 * fade * flicker;
    let base_color = Color::srgba(r, g, b, alpha);

    let emissive_strength = p.emissive_base + p.emissive_green_factor * g;
    let emissive = LinearRgba::new(
        r * emissive_strength,
        g * emissive_strength,
        b * p.emissive_blue_mult,
        0.0,
    );

    (base_color, emissive)
}

/// Returns fire or poop color based on wizard type.
pub fn effect_color_at(time: f32, fade: f32, is_excremage: bool) -> (Color, LinearRgba) {
    let palette = if is_excremage {
        &POOP_PALETTE
    } else {
        &FIRE_PALETTE
    };
    animated_color_at(time, fade, palette)
}

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

/// Moves, shrinks, and despawns fire spark particles.
pub fn update_fire_sparks(
    mut commands: Commands,
    mut spark_query: Query<(Entity, &mut FireSpark, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut spark, mut transform) in spark_query.iter_mut() {
        spark.time_alive += dt;

        if spark.time_alive >= constants::SPARK_LIFETIME {
            commands.entity(entity).try_despawn();
            continue;
        }

        transform.translation += spark.velocity * dt;

        // Apply gravity to sparks
        transform.translation.y -= 200.0 * dt * spark.time_alive;

        let remaining = 1.0 - (spark.time_alive / constants::SPARK_LIFETIME);
        let size = constants::SPARK_SIZE * remaining;
        transform.scale = Vec3::splat(size);
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
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU + time_secs * 3.7;
        // Random-ish elevation between 10° and 60° above horizontal
        let elevation = 0.2 + (i as f32 * 1.618).fract() * 0.8;
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
            },
            Mesh3d(assets.particle_quad.clone()),
            MeshMaterial3d(assets.fire_spark.clone()),
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
                base_size: constants::SHIMMER_SIZE,
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
