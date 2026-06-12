use super::super::components::{BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam};
use super::super::constants;
use crate::game::units::wizard::components::Wizard;
use bevy::prelude::*;

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
        .unwrap_or(constants::ECLIPSE_FALLBACK_SPELL_RANGE);

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
