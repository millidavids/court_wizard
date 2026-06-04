//! Disintegrate beam spawning, visuals, and cleanup.

use super::super::super::components::Wizard;
use super::casting::TalentConfig;
use super::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, BeamSmoke, DisintegrateBeam, DisintegrateParticle,
    SearingFinaleDetonation,
};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    Health, Hitbox, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::utils::{UniqueHitTracker, local_player_team};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_beam_with_damage(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    damage_per_tick: f32,
    talent_cfg: Option<&TalentConfig>,
    mini_spell_scale: f32,
    fan_offset_angle: f32,
) -> Entity {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.damage_per_tick_override = Some(damage_per_tick);
    if let Some(cfg) = talent_cfg {
        beam.width_multiplier = cfg.width_multiplier;
        beam.damage_multiplier = cfg.damage_multiplier;
        beam.escalating = cfg.escalating;
        beam.resonance = cfg.resonance;
    }
    beam.mini_spell_scale = mini_spell_scale;
    beam.fan_offset_angle = fan_offset_angle;
    beam.ground_collision = true;
    // Crystal beams only get the core beam mesh — no glow, flare, or eclipse.
    spawn_beam_core(commands, assets, beam)
}

/// Spawns a beam with talent configuration applied.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_beam_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    cfg: &TalentConfig,
    fan_offset_angle: f32,
    annihilation_cast_pos: Vec3,
    annihilation_sweep_dir: Vec3,
) {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.width_multiplier = cfg.width_multiplier;
    beam.damage_multiplier = cfg.damage_multiplier;
    beam.fan_offset_angle = fan_offset_angle;
    beam.escalating = cfg.escalating;
    beam.sweeping = cfg.sweeping;
    beam.searing_finale = cfg.searing_finale;
    beam.resonance = cfg.resonance;
    beam.annihilation = cfg.annihilation;
    beam.annihilation_cast_pos = annihilation_cast_pos;
    if cfg.sweeping {
        if cfg.annihilation {
            // For sky beams, sweep_center_direction stores the XZ forward reference
            beam.sweep_center_direction = annihilation_sweep_dir;
        } else {
            beam.sweep_center_direction = direction;
        }
        beam.sweep_direction = 1.0;
    }
    spawn_beam_visuals(commands, assets, beam);
}

/// Spawns only the core beam entity (mesh + component). Used by crystal beams
/// which don't need glow, flare, or eclipse siblings.
fn spawn_beam_core(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);

    commands
        .spawn((
            beam,
            UniqueHitTracker::default(),
            Mesh3d(assets.disintegrate_cone.clone()),
            MeshMaterial3d(assets.disintegrate_beam.clone()),
            Transform::from_translation(midpoint),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns the core beam entity plus glow, flare, and eclipse siblings.
/// Used by wizard-cast disintegrate beams.
fn spawn_beam_visuals(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);
    let beam_entity = spawn_beam_core(commands, assets, beam);

    // Glow cone sibling (wider, semi-transparent)
    commands.spawn((
        BeamGlow { beam_entity },
        Mesh3d(assets.disintegrate_cone.clone()),
        MeshMaterial3d(assets.disintegrate_glow.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Origin flare sphere
    commands.spawn((
        BeamOriginFlare { beam_entity },
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(assets.disintegrate_flare.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Ground eclipse at beam impact point
    commands.spawn((
        BeamEclipse { beam_entity },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.disintegrate_eclipse.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.05, 0.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        OnGameplayScreen,
    ));

    beam_entity
}

/// Helper to despawn all beam-related visual entities.
pub(super) fn despawn_all_beam_visuals(
    commands: &mut Commands,
    beams: &Query<(Entity, &mut DisintegrateBeam), Without<CrystalSpawn>>,
    glow_query: &Query<Entity, With<BeamGlow>>,
    flare_query: &Query<Entity, With<BeamOriginFlare>>,
    particle_query: &Query<Entity, With<DisintegrateParticle>>,
    eclipse_query: &Query<Entity, With<BeamEclipse>>,
) {
    for (entity, _) in beams.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in glow_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in flare_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in particle_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in eclipse_query.iter() {
        commands.entity(entity).try_despawn();
    }
}

/// System that updates beam cylinder transform to match beam data,
/// with pulsing width and color cycling.
pub fn update_beam_visuals(
    mut beam_query: Query<(
        &DisintegrateBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (beam, mut transform, material_handle) in beam_query.iter_mut() {
        let current_len = beam.current_length();
        // Crystal beams (ground_collision) shouldn't overshoot past their range.
        let overshoot = if beam.ground_collision {
            0.0
        } else {
            constants::BEAM_VISUAL_OVERSHOOT
        };
        let visual_len = current_len + overshoot;

        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.translation = beam.origin + beam.direction * visual_len / 2.0;

        // Pulsing width — use beam.beam_width() which includes talent multipliers
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let beam_width = beam.beam_width() * pulse * 0.7;
        transform.scale = Vec3::new(beam_width, visual_len, beam_width);

        // Color cycling: orange -> yellow -> white -> yellow -> orange
        if let Some(mat) = materials.get_mut(material_handle) {
            let cycle = (t * constants::COLOR_CYCLE_SPEED).sin() * 0.5 + 0.5; // 0..1
            // Interpolate emissive: orange(3,1.5,0.2) -> white(5,4.5,4)
            let r = 3.0 + cycle * 2.0;
            let g = 1.5 + cycle * 3.0;
            let b = 0.2 + cycle * 3.8;
            mat.emissive = bevy::color::LinearRgba::new(r, g, b, 1.0);

            // Also shift base color slightly
            let base_r = 1.0;
            let base_g = 0.6 + cycle * 0.35;
            let base_b = 0.1 + cycle * 0.6;
            mat.base_color = Color::srgb(base_r, base_g, base_b);
        }
    }
}

/// System that positions and animates the outer glow cylinder to follow its beam.
pub fn update_beam_glow(
    mut glow_query: Query<(&BeamGlow, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(beam) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        let current_len = beam.current_length();
        let visual_len = current_len + constants::BEAM_VISUAL_OVERSHOOT;

        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.translation = beam.origin + beam.direction * visual_len / 2.0;

        // Glow pulse + shimmer jitter from incommensurate frequencies
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let shimmer = constants::SHIMMER_AMPLITUDE
            * ((t * constants::SHIMMER_FREQ_A).sin() + (t * constants::SHIMMER_FREQ_B).cos());
        let glow_width =
            beam.beam_width() * constants::GLOW_WIDTH_MULTIPLIER * (pulse + shimmer) * 0.7;
        transform.scale = Vec3::new(glow_width, visual_len, glow_width);
    }
}

/// System that positions and animates the origin flare sphere.
pub fn update_beam_origin_flare(
    mut flare_query: Query<(&BeamOriginFlare, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (flare, mut transform) in flare_query.iter_mut() {
        let Ok(beam) = beam_query.get(flare.beam_entity) else {
            continue;
        };

        // Annihilation beams originate from Y=2000 — hide the flare.
        if beam.annihilation {
            transform.scale = Vec3::ZERO;
            continue;
        }

        transform.translation = beam.origin;

        // Pulsing scale
        let pulse = 1.0
            + constants::FLARE_PULSE_AMPLITUDE
                * (t * constants::FLARE_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let radius = constants::FLARE_RADIUS * pulse;
        transform.scale = Vec3::splat(radius);
    }
}

/// System that positions and scales the ground eclipse at the beam's impact point.
///
/// The eclipse is an ellipse matching the shadow a sphere would cast onto the
/// ground plane. Its major axis stretches along the beam's ground projection by
/// `1 / sin(elevation)`. Pulses in sync with the beam.
pub fn update_beam_eclipse(
    mut eclipse_query: Query<(&BeamEclipse, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    wizard_query: Query<&Wizard>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    let spell_range = wizard_query
        .iter()
        .next()
        .map(|w| w.spell_range)
        .unwrap_or(500.0);

    for (eclipse, mut transform) in eclipse_query.iter_mut() {
        let Ok(beam) = beam_query.get(eclipse.beam_entity) else {
            continue;
        };

        // Hide eclipse when beam angle is too steep (nearly horizontal)
        if beam.direction.y.abs() < 0.15 {
            transform.scale = Vec3::ZERO;
            continue;
        }

        let Some((eclipse_center, major_axis, _minor_axis, clipped_major, minor_radius)) =
            beam.eclipse_geometry(spell_range)
        else {
            transform.scale = Vec3::ZERO;
            continue;
        };

        transform.translation = Vec3::new(eclipse_center.x, 2.0, eclipse_center.z);

        // Pulse in sync with the beam core
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();

        let major_pulsed = clipped_major * pulse;
        let minor_pulsed = minor_radius * pulse;

        // Orient the ellipse so major axis aligns with beam's ground projection
        let theta = major_axis.z.atan2(major_axis.x);

        // Lay circle flat (XY → XZ), then rotate around Y to align stretch
        transform.rotation =
            Quat::from_rotation_y(theta) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        transform.scale = Vec3::new(major_pulsed, minor_pulsed, 1.0);
    }
}

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

// ── Talent-specific systems ──────────────────────────────────────────

/// System that auto-sweeps beams with the Sweeping Destruction talent.
/// Oscillates the beam direction around the sweep_center_direction.
pub fn update_sweep_beams(mut beam_query: Query<&mut DisintegrateBeam>, time: Res<Time>) {
    let dt = time.delta_secs();

    for mut beam in beam_query.iter_mut() {
        if !beam.sweeping {
            continue;
        }

        // Advance sweep angle
        beam.sweep_angle += constants::SWEEP_SPEED * beam.sweep_direction * dt;

        // Reverse direction at arc limits
        if beam.sweep_angle.abs() > constants::SWEEP_HALF_ARC {
            beam.sweep_angle = beam
                .sweep_angle
                .clamp(-constants::SWEEP_HALF_ARC, constants::SWEEP_HALF_ARC);
            beam.sweep_direction *= -1.0;
        }

        if beam.annihilation {
            // Sky beam: sweep origin position in XZ instead of rotating direction
            let forward = beam.sweep_center_direction;
            let perp = Vec3::new(-forward.z, 0.0, forward.x);
            let offset = perp * beam.sweep_angle * constants::ANNIHILATION_SWEEP_RADIUS;
            beam.origin = Vec3::new(
                beam.annihilation_cast_pos.x + offset.x,
                constants::ANNIHILATION_SKY_HEIGHT,
                beam.annihilation_cast_pos.z + offset.z,
            );
        } else {
            // Normal beam: apply sweep rotation to center direction
            let total_angle = beam.sweep_angle + beam.fan_offset_angle;
            let rotated = Quat::from_axis_angle(Vec3::Y, total_angle) * beam.sweep_center_direction;
            beam.direction = rotated;
        }
    }
}

/// System that processes searing finale detonations.
/// Applies burst damage once along the detonation line, then fades and despawns.
pub fn update_searing_finale_detonations(
    mut commands: Commands,
    mut detonation_query: Query<(Entity, &mut SearingFinaleDetonation, &mut Transform)>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (Without<Wizard>, Without<SearingFinaleDetonation>),
    >,
    time: Res<Time>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let dt = time.delta_secs();

    for (det_entity, mut detonation, mut transform) in detonation_query.iter_mut() {
        detonation.time_alive += dt;

        if detonation.time_alive >= constants::SEARING_FINALE_DURATION {
            commands.entity(det_entity).try_despawn();
            continue;
        }

        // Apply damage once
        if !detonation.damage_applied {
            detonation.damage_applied = true;

            for (
                entity,
                target_transform,
                hitbox,
                mut health,
                mut temp_hp,
                has_spell_shield,
                team,
            ) in target_query.iter_mut()
            {
                let pos = target_transform.translation;
                let to_point = pos - detonation.origin;
                let proj = to_point.dot(detonation.direction);

                if proj < -hitbox.radius || proj > detonation.length + hitbox.radius {
                    continue;
                }

                let closest =
                    detonation.origin + detonation.direction * proj.clamp(0.0, detonation.length);
                let dist = pos.distance(closest);

                if dist <= detonation.half_width + hitbox.radius {
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        detonation.damage,
                        constants::DAMAGE_TYPE,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                }
            }
        }

        // Visual: expand width over duration
        let progress = detonation.time_alive / constants::SEARING_FINALE_DURATION;
        let visual_width = detonation.half_width * 2.0 * (1.0 + progress * 0.5);
        let alpha = 1.0 - progress;
        transform.scale = Vec3::new(
            visual_width * alpha,
            detonation.length,
            visual_width * alpha,
        );
    }
}

/// Spawns a searing finale detonation entity along a beam's path.
pub(super) fn spawn_searing_finale(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: &DisintegrateBeam,
) {
    let current_len = beam.current_length();
    if current_len < 1.0 {
        return;
    }

    let midpoint = beam.origin + beam.direction * (current_len / 2.0);
    let half_width = beam.beam_width() * constants::SEARING_FINALE_WIDTH_MULT;
    let damage =
        beam.damage_per_tick() / constants::DAMAGE_INTERVAL * constants::SEARING_FINALE_DAMAGE_MULT;

    let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

    commands.spawn((
        SearingFinaleDetonation {
            origin: beam.origin,
            direction: beam.direction,
            length: current_len,
            half_width,
            damage,
            time_alive: 0.0,
            damage_applied: false,
        },
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(assets.searing_finale.clone()),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(half_width * 2.0, current_len, half_width * 2.0)),
        OnGameplayScreen,
    ));
}
