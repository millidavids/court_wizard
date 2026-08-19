use bevy::prelude::*;

use super::super::components::*;
use crate::game::components::Velocity;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{CombatAnimation, Corpse, FacingDirection, WalkingAnimation};

/// Overrides the ogre's facing direction to strongly prefer forward/backward.
/// Runs after the shared `update_facing_direction` to correct left/right
/// picks when the ogre is moving at a slight angle.
///
/// Filters on `With<OgreEnrageState>` (an ogre-only marker) — `With<Boss>`
/// would also match hags / dark mage / ray, which have their own facing logic
/// and would otherwise have their hysteresis-buffered facing clobbered each
/// frame by this raw-velocity override.
pub fn update_ogre_facing(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Boss>)>,
    mut bosses: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<OgreEnrageState>,
            Without<Corpse>,
            Without<CombatAnimation>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let cam_forward = camera_transform.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);

    for (velocity, mut facing, anim, material_handle) in &mut bosses {
        let vel_xz = Vec3::new(velocity.x, 0.0, velocity.z);
        if vel_xz.length_squared() < crate::game::units::components::ANIMATION_MOVE_THRESHOLD_SQ {
            continue;
        }

        let forward_dot = vel_xz.dot(cam_forward_xz);
        let right_dot = vel_xz.dot(cam_right);

        // Strong forward/back bias: only use left/right if the lateral component
        // is more than 3x the forward component
        let new_facing = if right_dot.abs() > forward_dot.abs() * 3.0 {
            if right_dot > 0.0 {
                FacingDirection::Right
            } else {
                FacingDirection::Left
            }
        } else if forward_dot < 0.0 {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mut mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
        }
    }
}
