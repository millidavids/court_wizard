use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::RayAssets;
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::components::OriginalMaterial;

pub fn update_ray_eye_movement(
    time: Res<Time>,
    mut commands: Commands,
    body_query: Query<(&Transform, &RayEyeState), (With<Ray>, Without<RayEye>)>,
    mut eyes: Query<
        (
            Entity,
            &mut Transform,
            &mut RayEye,
            &MeshMaterial3d<StandardMaterial>,
            Has<OriginalMaterial>,
        ),
        Without<Ray>,
    >,
    ray_assets: Res<RayAssets>,
    mut game_rng: ResMut<GameRng>,
) {
    let delta = time.delta_secs();
    let Ok((body_transform, eye_state)) = body_query.single() else {
        return;
    };
    let body_pos = body_transform.translation;

    // Collect positions for eye-to-eye separation
    let eye_positions: Vec<(Entity, Vec2)> = eyes
        .iter()
        .map(|(e, tf, _, _, _)| (e, Vec2::new(tf.translation.x, tf.translation.z)))
        .collect();

    for (entity, mut transform, mut eye, material, has_original_material) in &mut eyes {
        let i = eye.eye_type.index();
        // Only swap active/inactive material when no spell effect is tinting us.
        // The OriginalMaterial component indicates a spell effect owns the material.
        if !has_original_material {
            let new_handle = if eye_state.active[i] {
                &ray_assets.eye_materials[i]
            } else {
                &ray_assets.eye_inactive_material
            };
            if material.0 != *new_handle {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(new_handle.clone()));
            }
        }

        let my_pos = Vec2::new(transform.translation.x, transform.translation.z);

        // Fear eye: orbit Ray at a fixed radius instead of wandering
        if eye.eye_type == RayEyeType::Fear && eye_state.active[RayEyeType::Fear.index()] {
            // Use heading.x as orbit angle storage
            eye.heading.x += FEAR_EYE_ORBIT_SPEED * delta;
            let angle = eye.heading.x;
            let body_xz = Vec2::new(body_pos.x, body_pos.z);
            let orbit_pos = body_xz + Vec2::new(angle.cos(), angle.sin()) * FEAR_EYE_ORBIT_RADIUS;
            transform.translation.x = orbit_pos.x;
            transform.translation.z = orbit_pos.y;
            transform.translation.y = RAY_EYE_FLOAT_HEIGHT;
            continue;
        }

        // Separation force from other eyes
        let mut separation = Vec2::ZERO;
        for &(other_entity, other_pos) in &eye_positions {
            if other_entity == entity {
                continue;
            }
            let diff = my_pos - other_pos;
            let dist = diff.length();
            if dist > 0.001 && dist < RAY_EYE_SEPARATION_RADIUS {
                let push = (1.0 - dist / RAY_EYE_SEPARATION_RADIUS) * RAY_EYE_SEPARATION_FORCE;
                separation += (diff / dist) * push;
            }
        }

        // Steer away from Ray when close, toward Ray when far.
        // my_pos is a Vec2 in XZ world space (x→x, y→z), so my_pos.y is world-Z.
        let to_body = Vec2::new(body_pos.x - my_pos.x, body_pos.z - my_pos.y);
        let dist_to_body = to_body.length();
        let orbit_threshold = RAY_EYE_RADIUS * 4.0;

        let target_dir = if dist_to_body < 0.001 {
            eye.heading
        } else if dist_to_body < orbit_threshold {
            // Too close — steer away
            -(to_body / dist_to_body)
        } else {
            // Too far — steer back
            to_body / dist_to_body
        };

        let current_dir = eye.heading.normalize_or_zero();

        let dot = current_dir.dot(target_dir).clamp(-1.0, 1.0);
        let angle_to_target = dot.acos();
        let max_turn = RAY_EYE_TURN_RATE * delta;

        let new_dir = if angle_to_target <= max_turn || current_dir == Vec2::ZERO {
            target_dir
        } else {
            let cross = current_dir.x * target_dir.y - current_dir.y * target_dir.x;
            let turn_sign = if cross >= 0.0 { 1.0 } else { -1.0 };
            let cos_t = max_turn.cos();
            let sin_t = max_turn.sin() * turn_sign;
            Vec2::new(
                current_dir.x * cos_t - current_dir.y * sin_t,
                current_dir.x * sin_t + current_dir.y * cos_t,
            )
            .normalize_or_zero()
        };

        // Random angular drift so eyes don't converge on the same path
        let rng = &mut game_rng.0;
        let drift = (rng.random::<f32>() - 0.5) * RAY_EYE_DRIFT_RATE * delta;
        let cos_d = drift.cos();
        let sin_d = drift.sin();
        let drifted = Vec2::new(
            new_dir.x * cos_d - new_dir.y * sin_d,
            new_dir.x * sin_d + new_dir.y * cos_d,
        )
        .normalize_or_zero();

        eye.heading = drifted;

        // Move at constant speed + separation push
        transform.translation.x += (drifted.x * RAY_EYE_WANDER_SPEED + separation.x) * delta;
        transform.translation.z += (drifted.y * RAY_EYE_WANDER_SPEED + separation.y) * delta;

        // Pull back if too far from body
        if dist_to_body > RAY_EYE_WANDER_RADIUS {
            let pull = (dist_to_body - RAY_EYE_WANDER_RADIUS) * 2.0 * delta;
            let pull_dir = to_body / dist_to_body;
            transform.translation.x += pull_dir.x * pull;
            transform.translation.z += pull_dir.y * pull;
        }

        transform.translation.y = RAY_EYE_FLOAT_HEIGHT;
    }
}
