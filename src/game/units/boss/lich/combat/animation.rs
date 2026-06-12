use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::LichAssets;
use crate::game::components::Velocity;
use crate::game::units::boss::utils::{back_facing_for_velocity, sinusoidal_bob};
use crate::game::units::components::{Corpse, FacingDirection, WalkingAnimation};

/// Swaps the Lich's bound material to the casting sheet on the frame
/// `LichCasting` is inserted.
pub(crate) fn on_lich_cast_started(
    lich_assets: Res<LichAssets>,
    mut added: Query<&mut MeshMaterial3d<StandardMaterial>, (With<Lich>, Added<LichCasting>)>,
) {
    for mut mat in &mut added {
        mat.0 = lich_assets.casting_material.clone();
    }
}

/// Swaps the Lich's bound material back to the floating sheet on the frame
/// `LichCasting` is removed. Split from `on_lich_cast_started` so each system
/// has a single non-conflicting `&mut MeshMaterial3d` query.
pub(crate) fn on_lich_cast_ended(
    lich_assets: Res<LichAssets>,
    mut removed: RemovedComponents<LichCasting>,
    mut lich_query: Query<&mut MeshMaterial3d<StandardMaterial>, With<Lich>>,
) {
    for entity in removed.read() {
        if let Ok(mut mat) = lich_query.get_mut(entity) {
            mat.0 = lich_assets.floating_material.clone();
        }
    }
}

pub(crate) fn update_lich_facing(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut lich_query: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (With<Lich>, Without<Corpse>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(cam) = camera_query.single() else {
        return;
    };
    let cam_forward = cam.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();

    for (velocity, mut facing, anim, material_handle) in &mut lich_query {
        let v = Vec3::new(velocity.x, 0.0, velocity.z);
        let Some(new_facing) =
            back_facing_for_velocity(v, cam_forward_xz, LICH_BACK_FACING_THRESHOLD)
        else {
            continue;
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
        }
    }
}

pub(crate) fn update_lich_float(
    time: Res<Time>,
    mut lich_query: Query<(&LichFloatBase, &mut Transform), (With<Lich>, Without<Corpse>)>,
) {
    let bob = sinusoidal_bob(
        LICH_FLOAT_AMPLITUDE,
        LICH_FLOAT_FREQUENCY_HZ,
        time.elapsed_secs(),
    );
    for (base, mut transform) in &mut lich_query {
        transform.translation.y = base.base_y + LICH_FLOAT_BASE_OFFSET + bob;
    }
}
