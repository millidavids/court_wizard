//! Visual/animation systems: beam visuals, glow, flare, necrotic veins, pulse rings.

use super::super::components::*;
use super::super::constants;
use crate::game::units::constants::{EXCREMAGE_BROWN, EXCREMAGE_BROWN_DARK};
use bevy::prelude::*;

/// Updates necrotic explosion burst visuals — expand and fade.
pub fn update_necrotic_explosion_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(
        Entity,
        &mut NecroticExplosionBurst,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut burst, mut transform, material_handle) in bursts.iter_mut() {
        burst.time_alive += dt;

        if burst.time_alive >= burst.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = burst.time_alive / burst.lifetime;
        let radius = burst.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let color = constants::NECROTIC_EXPLOSION_COLOR.to_srgba();
            let alpha = 0.6 * (1.0 - progress);
            material.base_color = Color::srgba(color.red, color.green, color.blue, alpha);
        }
    }
}

/// Updates Finger of Death beam visuals based on cast progress and fire state.
pub fn update_finger_of_death_beam_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &mut FingerOfDeathBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<crate::config::GameConfig>,
) {
    let t = time.elapsed_secs();
    let is_excremage = config.wizard_type == crate::config::WizardType::Excremage;

    for (mut beam, mut transform, material_handle) in beam_query.iter_mut() {
        if beam.has_fired {
            beam.time_since_fired += time.delta_secs();
        }

        let visual_len = beam.length + constants::BEAM_VISUAL_OVERSHOOT;

        // Position at origin, rotate to point along beam direction
        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Pulsing width
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let beam_width = base_width * pulse;

        // During cast: scale up with cast_progress; after fire: full size
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::new(
            beam_width * progress_scale,
            visual_len * progress_scale,
            beam_width * progress_scale,
        );

        // Color cycling and fade
        if let Some(mat) = materials.get_mut(&material_handle.0) {
            if beam.has_fired {
                let fade = (1.0 - beam.time_since_fired / constants::POST_FIRE_DURATION).max(0.0);

                if is_excremage {
                    let srgba = EXCREMAGE_BROWN.to_srgba();
                    mat.base_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
                    mat.emissive = LinearRgba::new(
                        srgba.red * 2.0 * fade,
                        srgba.green * 0.5 * fade,
                        srgba.blue * 0.2 * fade,
                        1.0,
                    );
                } else {
                    // Purple color cycling: dark purple -> bright violet -> pale purple
                    let cycle = (t * constants::COLOR_CYCLE_SPEED).sin() * 0.5 + 0.5;
                    let r = (0.5 + cycle * 0.5) * fade;
                    let g = (0.0 + cycle * 0.3) * fade;
                    let b = (0.6 + cycle * 0.4) * fade;
                    mat.base_color = Color::srgb(r, g, b);
                    mat.emissive = LinearRgba::new(
                        (1.5 + cycle * 1.5) * fade,
                        (0.0 + cycle * 0.8) * fade,
                        (2.5 + cycle * 2.0) * fade,
                        1.0,
                    );
                }
            } else {
                // During cast: growing intensity with cast_progress
                let intensity = beam.cast_progress;

                if is_excremage {
                    let srgba = EXCREMAGE_BROWN_DARK.to_srgba();
                    mat.base_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
                    mat.emissive = LinearRgba::new(
                        srgba.red * intensity,
                        srgba.green * 0.3 * intensity,
                        srgba.blue * 0.1 * intensity,
                        1.0,
                    );
                } else {
                    mat.base_color = Color::srgb(0.6 * intensity, 0.0, 0.8 * intensity);
                    mat.emissive = LinearRgba::new(1.5 * intensity, 0.0, 2.5 * intensity, 1.0);
                }
            }
        }
    }
}

/// Updates necrotic vein particles: meander, fade color, shrink, despawn when expired.
pub fn update_necrotic_veins(
    mut commands: Commands,
    time: Res<Time>,
    mut veins: Query<(
        Entity,
        &mut NecroticVein,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut vein, mut transform, material_handle) in veins.iter_mut() {
        vein.time_alive += dt;

        if vein.time_alive >= vein.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = vein.time_alive / vein.lifetime;

        // Meander: apply lateral sine-wave offset to velocity direction
        let wander_offset = (t * constants::VEIN_WANDER_FREQUENCY + vein.wander_phase).sin()
            * constants::VEIN_WANDER_AMPLITUDE
            * dt;
        let lateral = Vec3::new(-vein.velocity.z, 0.0, vein.velocity.x).normalize_or_zero();
        transform.translation += vein.velocity * dt + lateral * wander_offset;

        // Clamp Y to ground level
        transform.translation.y = constants::VEIN_Y_POSITION;

        // Shrink over lifetime
        let scale = vein.base_size * (1.0 - progress);
        transform.scale = Vec3::splat(scale);

        // Animate material: purple → dark purple → black, alpha fading out
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let r = 0.6 * (1.0 - progress);
            let g = 0.0;
            let b = 0.8 * (1.0 - progress * 0.5);
            let alpha = 0.8 * (1.0 - progress);
            material.base_color = Color::srgba(r, g, b, alpha);
            let em_scale = 1.0 - progress;
            material.emissive = LinearRgba::new(1.5 * em_scale, 0.0, 2.0 * em_scale, 1.0);
        }
    }
}

/// Updates the glow aura to follow its beam with shimmer and pulsing.
pub fn update_finger_of_death_glow(
    mut glow_query: Query<(&FingerOfDeathGlow, &mut Transform), Without<FingerOfDeathBeam>>,
    beam_query: Query<&FingerOfDeathBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(beam) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        let visual_len = beam.length + constants::BEAM_VISUAL_OVERSHOOT;

        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Glow pulse + shimmer jitter
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let shimmer = constants::SHIMMER_AMPLITUDE
            * ((t * constants::SHIMMER_FREQ_A).sin() + (t * constants::SHIMMER_FREQ_B).cos());
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let glow_width = base_width * constants::GLOW_WIDTH_MULTIPLIER * (pulse + shimmer);

        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::new(
            glow_width * progress_scale,
            visual_len * progress_scale,
            glow_width * progress_scale,
        );
    }
}

/// Despawns glow entities when their beam no longer exists.
pub fn cleanup_finger_of_death_glow(
    mut commands: Commands,
    glow_query: Query<(Entity, &FingerOfDeathGlow)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (glow_entity, glow) in glow_query.iter() {
        if beam_query.get(glow.beam_entity).is_err() {
            commands.entity(glow_entity).try_despawn();
        }
    }
}

/// Updates the origin flare to pulse at the beam origin.
pub fn update_finger_of_death_flare(
    mut flare_query: Query<(&FingerOfDeathFlare, &mut Transform)>,
    beam_query: Query<&FingerOfDeathBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (flare, mut transform) in flare_query.iter_mut() {
        let Ok(beam) = beam_query.get(flare.beam_entity) else {
            continue;
        };

        transform.translation = beam.origin;

        let pulse = 1.0
            + constants::FLARE_PULSE_AMPLITUDE
                * (t * constants::FLARE_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let radius = constants::FLARE_RADIUS * pulse;

        // Scale with cast progress so flare grows during cast
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::splat(radius * progress_scale);
    }
}

/// Despawns flare entities when their beam no longer exists.
pub fn cleanup_finger_of_death_flare(
    mut commands: Commands,
    flare_query: Query<(Entity, &FingerOfDeathFlare)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (flare_entity, flare) in flare_query.iter() {
        if beam_query.get(flare.beam_entity).is_err() {
            commands.entity(flare_entity).try_despawn();
        }
    }
}

/// Updates necrotic pulse ring: expand scale, fade alpha, despawn when expired.
pub fn update_necrotic_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(
        Entity,
        &mut NecroticPulse,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut pulse, mut transform, material_handle) in pulses.iter_mut() {
        pulse.time_alive += dt;

        if pulse.time_alive >= pulse.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = pulse.time_alive / pulse.lifetime;

        // Expand from small to max_radius
        let radius = pulse.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        // Fade alpha from 0.5 to 0
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 0.5 * (1.0 - progress);
            material.base_color = Color::srgba(0.4, 0.0, 0.6, alpha);
        }
    }
}
