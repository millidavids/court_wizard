use bevy::math::Affine2;
use bevy::prelude::*;

use super::super::combat::enrage_phase_tint;
use super::super::components::*;
use super::super::constants::*;
use super::super::resources::OgreAssets;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{CombatAnimation, Corpse, FacingDirection, WalkingAnimation};

/// Updates ogre sprite visuals during charge attack phases.
/// Swaps to attacking texture, sets the correct frame, applies red flash and vibration.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_ogre_charge_visuals(
    time: Res<Time>,
    ogre_assets: Res<OgreAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Boss>)>,
    mut bosses: Query<
        (
            Entity,
            &mut Transform,
            &OgreChargeState,
            &OgreEnrageState,
            &MeshMaterial3d<StandardMaterial>,
            &WalkingAnimation,
            &mut FacingDirection,
            Option<&mut OgreChargeVisuals>,
        ),
        (With<Boss>, Without<Corpse>, Without<Camera3d>),
    >,
) {
    let cam_forward_xz = camera_query
        .single()
        .ok()
        .map(|cam| {
            let fwd = cam.forward();
            Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero()
        })
        .unwrap_or(Vec3::NEG_Z);

    let delta = time.delta_secs();

    for (
        entity,
        mut transform,
        charge_state,
        enrage_state,
        material_handle,
        walking_anim,
        mut facing,
        charge_visuals,
    ) in &mut bosses
    {
        match charge_state {
            OgreChargeState::Telegraphing {
                elapsed, direction, ..
            } => {
                // Set facing direction from charge direction
                let new_facing = facing_from_world_direction(*direction, cam_forward_xz);
                *facing = new_facing;

                if let Some(mut visuals) = charge_visuals {
                    // Ongoing telegraph — update effects
                    visuals.elapsed += delta;
                    let progress = (*elapsed / OGRE_CHARGE_TELEGRAPH_DURATION).min(1.0);

                    if let Some(mut mat) = materials.get_mut(&material_handle.0) {
                        // First frame: swap texture to attacking sheet
                        if !visuals.texture_swapped {
                            mat.base_color_texture = Some(ogre_assets.attacking_texture.clone());
                            visuals.texture_swapped = true;
                        }

                        // Show frame 0 (wind-up) in correct direction
                        let row = OGRE_ATTACKING_DIRECTION_ROWS[new_facing as usize];
                        mat.uv_transform = ogre_frame_uv_transform(0, row);

                        // Red flash: pulse between enrage tint and flash color
                        let flash_t =
                            (visuals.elapsed * OGRE_CHARGE_FLASH_FREQUENCY * std::f32::consts::TAU)
                                .sin()
                                * 0.5
                                + 0.5;
                        let base_tint = enrage_phase_tint(enrage_state.phase);
                        mat.base_color = Color::LinearRgba(
                            base_tint
                                .to_linear()
                                .mix(&OGRE_CHARGE_FLASH_COLOR.to_linear(), flash_t),
                        );
                    }

                    // Vibration: sinusoidal offset scaled by progress
                    let amp = OGRE_CHARGE_VIBRATION_AMPLITUDE * progress;
                    let vib_x =
                        (visuals.elapsed * OGRE_CHARGE_VIBRATION_FREQ_X * std::f32::consts::TAU)
                            .sin()
                            * amp;
                    let vib_z =
                        (visuals.elapsed * OGRE_CHARGE_VIBRATION_FREQ_Z * std::f32::consts::TAU)
                            .sin()
                            * amp;
                    transform.translation.x = visuals.base_position.x + vib_x;
                    transform.translation.z = visuals.base_position.z + vib_z;
                } else {
                    // First frame of telegraph — insert visuals component
                    // and remove any active combat/throw animations
                    commands
                        .entity(entity)
                        .remove::<CombatAnimation>()
                        .remove::<OgreThrowWindup>()
                        .insert(OgreChargeVisuals {
                            texture_swapped: false,
                            elapsed: 0.0,
                            base_position: transform.translation,
                        });
                }
            }

            OgreChargeState::Charging { direction, .. } => {
                if let Some(mut visuals) = charge_visuals
                    && let Some(mut mat) = materials.get_mut(&material_handle.0)
                {
                    // Restore base position on first charging frame
                    // (remove vibration offset before charge movement begins)
                    if visuals.elapsed > 0.0 {
                        transform.translation.x = visuals.base_position.x;
                        transform.translation.z = visuals.base_position.z;
                        visuals.elapsed = 0.0;
                    }

                    // Show frame 1 (charge pose)
                    let new_facing = facing_from_world_direction(*direction, cam_forward_xz);
                    *facing = new_facing;
                    let row = OGRE_ATTACKING_DIRECTION_ROWS[new_facing as usize];
                    mat.uv_transform = ogre_frame_uv_transform(1, row);

                    // Restore normal tint (stop red flash)
                    mat.base_color = enrage_phase_tint(enrage_state.phase);
                }
            }

            OgreChargeState::Recovery { .. } => {
                if charge_visuals.is_some()
                    && let Some(mut mat) = materials.get_mut(&material_handle.0)
                {
                    let row = OGRE_ATTACKING_DIRECTION_ROWS[*facing as usize];
                    mat.uv_transform = ogre_frame_uv_transform(2, row);
                }
            }

            OgreChargeState::Idle { .. } | OgreChargeState::Targeting => {
                // Cleanup: restore walking texture and remove visuals
                if let Some(visuals) = charge_visuals {
                    if visuals.texture_swapped
                        && let Some(mut mat) = materials.get_mut(&material_handle.0)
                    {
                        mat.base_color_texture = Some(ogre_assets.walking_texture.clone());
                        mat.base_color = enrage_phase_tint(enrage_state.phase);
                        // Reset UV to walking idle frame
                        mat.uv_transform = walking_anim.uv_transform(*facing);
                    }
                    // Only restore base position if vibration was still active
                    // (CC interruption during telegraph). After charging starts,
                    // elapsed is reset to 0 and the ogre has moved legitimately.
                    if visuals.elapsed > 0.0 {
                        transform.translation.x = visuals.base_position.x;
                        transform.translation.z = visuals.base_position.z;
                    }
                    commands.entity(entity).remove::<OgreChargeVisuals>();
                }
            }
        }
    }
}

/// Creates a CombatAnimation configured for the ogre's sprite sheet dimensions.
pub(crate) fn ogre_combat_animation(
    direction_rows: [usize; 4],
    combat_texture: Handle<Image>,
    walking_texture: Handle<Image>,
) -> CombatAnimation {
    CombatAnimation {
        current_frame: 0,
        elapsed: 0.0,
        columns: OGRE_SPRITE_COLUMNS,
        frame_uv: OGRE_FRAME_UV,
        direction_rows,
        combat_texture,
        walking_texture,
        started: false,
    }
}

/// Returns the UV transform for a specific frame and direction row in the ogre sprite sheet.
fn ogre_frame_uv_transform(frame: usize, direction_row: usize) -> Affine2 {
    let uv_offset = Vec2::new(
        frame as f32 * OGRE_FRAME_UV.x,
        direction_row as f32 * OGRE_FRAME_UV.y,
    );
    Affine2::from_scale_angle_translation(OGRE_FRAME_UV, 0.0, uv_offset)
}

/// Derives a FacingDirection from a world-space direction vector relative to the camera.
pub(crate) fn facing_from_world_direction(dir: Vec3, cam_forward_xz: Vec3) -> FacingDirection {
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);
    let forward_dot = dir.dot(cam_forward_xz);
    let right_dot = dir.dot(cam_right);
    if forward_dot.abs() > right_dot.abs() {
        if forward_dot < 0.0 {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        }
    } else if right_dot > 0.0 {
        FacingDirection::Right
    } else {
        FacingDirection::Left
    }
}
