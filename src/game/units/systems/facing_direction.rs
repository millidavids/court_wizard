use bevy::prelude::*;

use super::super::components::{
    ANIMATION_MOVE_THRESHOLD_SQ, Corpse, FacingDirection, FacingDwell, FacingHysteresisBoost,
    PolymorphedModifier, SmoothedFacingVelocity, WalkingAnimation,
};
use crate::game::components::Velocity;

/// Updates facing direction based on velocity relative to camera, updating UV row.
#[allow(clippy::type_complexity)]
pub fn update_facing_direction(
    camera_query: Query<&Transform, With<Camera3d>>,
    time: Res<Time>,
    mut unit_query: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
            Option<&FacingHysteresisBoost>,
            Option<&mut FacingDwell>,
            Option<&mut SmoothedFacingVelocity>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Camera forward/right on XZ plane
    let cam_forward = camera_transform.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);
    let dt = time.delta_secs();

    // Base angular buffer past the 45° axis boundary. With `buffer = 8°`,
    // a unit currently on the forward-back axis only switches to left-right
    // when the velocity is more than `45 + 8 = 53°` off-axis, and vice versa.
    // `FacingHysteresisBoost` widens the buffer further for jittery units.
    const BASE_BUFFER_DEG: f32 = 8.0;
    let default_buffer_ratio = (45.0_f32 + BASE_BUFFER_DEG).to_radians().tan();

    for (velocity, mut facing, anim, material_handle, hysteresis_boost, mut dwell, mut smoothed) in
        &mut unit_query
    {
        let raw_vel = Vec3::new(velocity.x, 0.0, velocity.z);

        // Update the smoothed velocity (low-pass filter) every frame, even when
        // the dwell is locking the facing — keeps the trend representative.
        let smoothed_vel = if let Some(s) = smoothed.as_mut() {
            let alpha = if s.time_constant > 0.0 {
                (dt / s.time_constant).clamp(0.0, 1.0)
            } else {
                1.0
            };
            s.velocity = s.velocity.lerp(raw_vel, alpha);
            s.velocity
        } else {
            raw_vel
        };

        // Tick dwell timer regardless of velocity so it expires while the unit
        // is briefly stationary; while non-zero the facing is locked in.
        if let Some(d) = dwell.as_mut() {
            d.time_remaining = (d.time_remaining - dt).max(0.0);
            if d.time_remaining > 0.0 {
                continue;
            }
        }

        if smoothed_vel.length_squared() < ANIMATION_MOVE_THRESHOLD_SQ {
            continue;
        }

        // Project the smoothed velocity onto camera axes.
        let forward_dot = smoothed_vel.dot(cam_forward_xz);
        let right_dot = smoothed_vel.dot(cam_right);
        let abs_fwd = forward_dot.abs();
        let abs_right = right_dot.abs();

        // Buffer ratio = tan(45° + buffer). At buffer=8°, ratio ≈ 1.327: the
        // new axis must dominate by 32.7% before we switch. Precomputed for the
        // default; boosted entities (rare) recompute per-entity.
        let buffer_ratio = match hysteresis_boost {
            Some(boost) if (boost.0 - 1.0).abs() > f32::EPSILON => (45.0_f32
                + BASE_BUFFER_DEG * boost.0)
                .clamp(45.0, 89.0)
                .to_radians()
                .tan(),
            _ => default_buffer_ratio,
        };

        let current_is_forward_back =
            matches!(*facing, FacingDirection::Forward | FacingDirection::Back);
        let on_forward_back = if current_is_forward_back {
            // Stay on FB axis unless |right| is more than buffer_ratio × |fwd|.
            abs_right < abs_fwd * buffer_ratio
        } else {
            // On LR axis: switch to FB only if |fwd| beats |right| by buffer_ratio.
            abs_fwd > abs_right * buffer_ratio
        };

        let new_facing = if on_forward_back {
            if forward_dot >= 0.0 {
                FacingDirection::Forward
            } else {
                FacingDirection::Back
            }
        } else if right_dot >= 0.0 {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mut mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
            if let Some(d) = dwell.as_mut() {
                d.time_remaining = d.duration;
            }
        }
    }
}
