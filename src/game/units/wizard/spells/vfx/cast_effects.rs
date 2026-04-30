//! Cast flares, motes, smoke poofs, dust, and aura bubbles.

use bevy::prelude::*;

use super::area_effects::spawn_fire_smoke_puff;
use super::components::{AuraBubbleVfx, CastFlare, FireSpark, FloatingMote, SmokePoof};
use super::constants;
use super::constants::UPWARD_ROTATION;
use crate::game::components::Billboard;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

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
        SpellSchool::Fire => (
            assets.flare_fire_glow.clone(),
            assets.flare_fire_spark.clone(),
        ),
        SpellSchool::Lightning => (
            assets.flare_lightning_glow.clone(),
            assets.flare_lightning_spark.clone(),
        ),
        SpellSchool::Arcane => (
            assets.flare_arcane_glow.clone(),
            assets.flare_arcane_spark.clone(),
        ),
        SpellSchool::Nature => (
            assets.flare_nature_glow.clone(),
            assets.flare_nature_spark.clone(),
        ),
        SpellSchool::Holy => (
            assets.flare_holy_glow.clone(),
            assets.flare_holy_spark.clone(),
        ),
        SpellSchool::Dark => (
            assets.flare_dark_glow.clone(),
            assets.flare_dark_spark.clone(),
        ),
        SpellSchool::Force => (
            assets.flare_force_glow.clone(),
            assets.flare_force_spark.clone(),
        ),
        SpellSchool::Transmutation => (
            assets.flare_transmutation_glow.clone(),
            assets.flare_transmutation_spark.clone(),
        ),
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
        let sway = (t * constants::MOTE_SWAY_FREQUENCY * std::f32::consts::TAU + mote.phase).sin()
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
            assets,
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

/// Spawns a temporary aura bubble that grows calmly then fades out.
pub fn spawn_aura_bubble(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: Handle<crate::game::units::wizard::spells::visual_assets::AuraSphereMaterial>,
    position: Vec3,
    max_radius: f32,
    duration: f32,
) {
    spawn_aura_bubble_inner(
        commands, assets, material, position, max_radius, duration, false,
    );
}

/// Spawns a contracting aura bubble that fades in at full size and shrinks to a point.
pub fn spawn_aura_bubble_contracting(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: Handle<crate::game::units::wizard::spells::visual_assets::AuraSphereMaterial>,
    position: Vec3,
    max_radius: f32,
    duration: f32,
) {
    spawn_aura_bubble_inner(
        commands, assets, material, position, max_radius, duration, true,
    );
}

fn spawn_aura_bubble_inner(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    material: Handle<crate::game::units::wizard::spells::visual_assets::AuraSphereMaterial>,
    position: Vec3,
    max_radius: f32,
    duration: f32,
    contracting: bool,
) {
    let initial_scale = if contracting { max_radius } else { 0.1 };
    commands.spawn((
        AuraBubbleVfx {
            max_radius,
            duration,
            time_alive: 0.0,
            contracting,
        },
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(Vec3::splat(initial_scale)),
        OnGameplayScreen,
    ));
}

/// Updates aura bubble VFX: grows or contracts depending on mode.
pub fn update_aura_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut bubbles: Query<(Entity, &mut AuraBubbleVfx, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (entity, mut bubble, mut transform) in &mut bubbles {
        bubble.time_alive += delta;

        if bubble.time_alive >= bubble.duration {
            commands.entity(entity).try_despawn();
            continue;
        }

        let t = bubble.time_alive / bubble.duration;

        let scale = if bubble.contracting {
            // Contracting: start at full size, shrink to zero with ease-in
            let shrink = 1.0 - t;
            let eased = shrink * shrink;
            bubble.max_radius * eased
        } else {
            // Expanding: grow with ease-out over first 40%, hold, then shrink over last 30%
            if t < 0.4 {
                let grow = t / 0.4;
                let eased = 1.0 - (1.0 - grow) * (1.0 - grow);
                bubble.max_radius * eased
            } else if t > 0.7 {
                let fade = 1.0 - (t - 0.7) / 0.3;
                bubble.max_radius * fade
            } else {
                bubble.max_radius
            }
        };

        transform.scale = Vec3::splat(scale.max(0.1));
    }
}
