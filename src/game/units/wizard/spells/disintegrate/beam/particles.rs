use super::super::components::{BeamSmoke, DisintegrateBeam, DisintegrateParticle};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// System that spawns impact particles at the beam tip (host / single-player).
pub fn spawn_impact_particles(
    mut commands: Commands,
    beam_query: Query<&DisintegrateBeam>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= constants::PARTICLE_SPAWN_INTERVAL;

    let elapsed = time.elapsed_secs();
    for beam in beam_query.iter() {
        emit_impact_particles(
            &mut commands,
            &visual_assets,
            elapsed,
            beam.origin,
            beam.direction,
            beam.current_length(),
            beam.annihilation,
        );
    }
}

/// Spawns one batch of impact particles at a beam tip from raw geometry.
///
/// Shared by the SP `spawn_impact_particles` system and the multiplayer guest
/// path (`crate::game::multiplayer::spell_sync::spawn_ghost_beam_impact_vfx`),
/// whose snapshot-driven ghost beam carries no `DisintegrateBeam` to query.
pub(crate) fn emit_impact_particles(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    elapsed_secs: f32,
    origin: Vec3,
    direction: Vec3,
    current_len: f32,
    annihilation: bool,
) {
    if current_len < 1.0 {
        return;
    }

    let tip = origin + direction * current_len;

    // Annihilation beams: skip particles when tip is still high in the sky (growth phase).
    if annihilation && tip.y > 50.0 {
        return;
    }

    // Build a perpendicular basis for random spread
    let up = if direction.y.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let right = direction.cross(up).normalize();
    let forward = right.cross(direction).normalize();

    for i in 0..constants::PARTICLE_COUNT_PER_SPAWN {
        // Spread particles in a circle perpendicular to the beam
        let angle = (i as f32 / constants::PARTICLE_COUNT_PER_SPAWN as f32) * std::f32::consts::TAU
            + elapsed_secs * 17.3; // rotating offset
        let spread = right * angle.cos() + forward * angle.sin();
        let velocity = spread * constants::PARTICLE_SPEED;

        commands.spawn((
            DisintegrateParticle {
                velocity,
                time_alive: 0.0,
            },
            Mesh3d(visual_assets.particle_quad.clone()),
            MeshMaterial3d(visual_assets.disintegrate_particle.clone()),
            Transform::from_translation(tip)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(constants::PARTICLE_SIZE)),
            OnGameplayScreen,
        ));
    }
}

/// System that moves, shrinks, and despawns impact particles.
pub fn update_impact_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut DisintegrateParticle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform) in particle_query.iter_mut() {
        particle.time_alive += dt;

        if particle.time_alive >= constants::PARTICLE_LIFETIME {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Move by velocity
        transform.translation += particle.velocity * dt;

        // Scale down linearly over lifetime
        let remaining = 1.0 - (particle.time_alive / constants::PARTICLE_LIFETIME);
        let size = constants::PARTICLE_SIZE * remaining;
        transform.scale = Vec3::splat(size);
    }
}

/// System that spawns dark smoke wisps along the beam (host / single-player).
/// Smoke is independent of the beam and self-dissipates after its lifetime.
pub fn spawn_beam_smoke(
    mut commands: Commands,
    beam_query: Query<&DisintegrateBeam>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();
    for beam in beam_query.iter() {
        emit_beam_smoke(
            &mut commands,
            &visual_assets,
            t,
            beam.origin,
            beam.direction,
            beam.current_length(),
            beam.annihilation,
        );
    }
}

/// Spawns one batch of smoke wisps plus a heat shimmer along a beam from raw
/// geometry. Shared by the SP `spawn_beam_smoke` system and the multiplayer
/// guest path (see `emit_impact_particles`).
pub(crate) fn emit_beam_smoke(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    t: f32,
    origin: Vec3,
    direction: Vec3,
    current_len: f32,
    annihilation: bool,
) {
    if current_len < 1.0 {
        return;
    }

    // Build perpendicular basis
    let up = if direction.y.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let right = direction.cross(up).normalize();
    let forward = right.cross(direction).normalize();

    for i in 0..constants::SMOKE_COUNT_PER_SPAWN {
        // Spawn at a random-ish point along the beam length
        let frac =
            ((i as f32 + 0.5) / constants::SMOKE_COUNT_PER_SPAWN as f32 + (t * 13.7).fract()) % 1.0;

        let pos = origin + direction * (current_len * frac);

        // Annihilation beams span from Y=2000 — only spawn smoke near ground level.
        if annihilation && pos.y > 50.0 {
            continue;
        }

        // Mostly upward drift with slight lateral spread
        let angle = (i as f32 * 2.39 + t * 7.1).sin(); // pseudo-random lateral
        let lateral = (right * angle.cos() + forward * angle.sin()) * constants::SMOKE_SPREAD_SPEED;
        let velocity = Vec3::Y * constants::SMOKE_RISE_SPEED + lateral;

        commands.spawn((
            BeamSmoke {
                velocity,
                time_alive: 0.0,
            },
            Mesh3d(visual_assets.particle_quad.clone()),
            MeshMaterial3d(visual_assets.disintegrate_smoke.clone()),
            Transform::from_translation(pos)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(constants::SMOKE_SIZE)),
            OnGameplayScreen,
        ));
    }

    // Heat shimmer along the beam
    let shimmer_frac = (t * 5.3 + 0.37).fract();
    let shimmer_pos = origin + direction * (current_len * shimmer_frac);

    // Annihilation beams: skip shimmer high in the sky
    if annihilation && shimmer_pos.y > 50.0 {
        return;
    }
    vfx::systems::spawn_heat_shimmer(commands, visual_assets, shimmer_pos, 1, t);
}

/// System that drifts, scale-fades, and despawns smoke wisps.
/// Runs independently of beam existence so smoke lingers after casting stops.
pub fn update_beam_smoke(
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut BeamSmoke, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut smoke, mut transform) in smoke_query.iter_mut() {
        smoke.time_alive += dt;

        if smoke.time_alive >= constants::SMOKE_LIFETIME {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Drift by velocity
        transform.translation += smoke.velocity * dt;

        // Grow then shrink: peak at 60% of lifetime, shrink to zero by end
        let progress = smoke.time_alive / constants::SMOKE_LIFETIME;
        let size = if progress < 0.6 {
            constants::SMOKE_SIZE * (1.0 + progress * 0.83)
        } else {
            let shrink = 1.0 - (progress - 0.6) / 0.4;
            constants::SMOKE_SIZE * 1.5 * shrink
        };
        transform.scale = Vec3::splat(size);
    }
}
