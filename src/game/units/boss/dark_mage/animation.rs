//! Dark mage sprite-sheet animation, casting/floating material swap, and
//! sinusoidal hover bob. Adapts the lich's pattern to the dark mage's
//! explicit `DarkMageState` enum and runs a custom continuous-frame ticker
//! so the float and cast animations keep cycling while the boss stands
//! still during long telegraph wind-ups.

use bevy::prelude::*;

use super::components::{DarkMage, DarkMageFloatBase, DarkMageState};
use super::constants::{
    DARK_MAGE_BACK_FACING_THRESHOLD, DARK_MAGE_FLOAT_AMPLITUDE, DARK_MAGE_FLOAT_FREQUENCY_HZ,
};
use super::resources::DarkMageAssets;
use crate::game::components::Velocity;
use crate::game::units::boss::utils::{back_facing_for_velocity, sinusoidal_bob};
use crate::game::units::components::{Corpse, FacingDirection, OriginalMaterial, WalkingAnimation};

pub(super) fn update_dark_mage_facing(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut mage_query: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(cam) = camera_query.single() else {
        return;
    };
    let cam_forward = cam.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();

    for (velocity, mut facing, anim, material_handle) in &mut mage_query {
        let v = Vec3::new(velocity.x, 0.0, velocity.z);
        let Some(new_facing) =
            back_facing_for_velocity(v, cam_forward_xz, DARK_MAGE_BACK_FACING_THRESHOLD)
        else {
            continue;
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mut mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
        }
    }
}

pub(super) fn update_dark_mage_animation(
    time: Res<Time>,
    mut anim_query: Query<
        (
            &mut WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
            &FacingDirection,
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    for (mut anim, material_handle, facing) in &mut anim_query {
        if anim.tick(delta)
            && let Some(mut mat) = materials.get_mut(material_handle)
        {
            mat.uv_transform = anim.uv_transform(*facing);
        }
    }
}

/// Plays the casting sheet during Telegraphing+Casting; otherwise the floating
/// sheet. Respects `OriginalMaterial` so freeze/petrify masks restore the
/// right sheet on thaw.
pub(super) fn sync_dark_mage_material(
    assets: Res<DarkMageAssets>,
    mut query: Query<
        (
            &DarkMageState,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&mut OriginalMaterial>,
        ),
        (With<DarkMage>, Changed<DarkMageState>),
    >,
) {
    for (state, mut mat, original) in &mut query {
        let target = if matches!(
            state,
            DarkMageState::Telegraphing { .. } | DarkMageState::Casting { .. }
        ) {
            assets.casting_material.clone()
        } else {
            assets.floating_material.clone()
        };

        if let Some(mut orig) = original {
            if orig.0 != target {
                orig.0 = target;
            }
        } else if mat.0 != target {
            mat.0 = target;
        }
    }
}

pub(super) fn update_dark_mage_float(
    time: Res<Time>,
    mut query: Query<(&DarkMageFloatBase, &mut Transform), (With<DarkMage>, Without<Corpse>)>,
) {
    let bob = sinusoidal_bob(
        DARK_MAGE_FLOAT_AMPLITUDE,
        DARK_MAGE_FLOAT_FREQUENCY_HZ,
        time.elapsed_secs(),
    );
    for (base, mut transform) in &mut query {
        transform.translation.y = base.base_y + bob;
    }
}
