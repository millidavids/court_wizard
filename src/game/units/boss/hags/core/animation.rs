use bevy::prelude::*;
use rand::Rng;

use super::super::constants::*;
use super::super::resources::HagAssets;
use crate::game::units::boss::utils::{EYE_FRAME_UV, EYE_PULSE_FRAME_DURATION, EYE_SHEET_COLUMNS};
use crate::game::units::components::{
    CombatAnimation, FacingDirection, PulsingAnimation, WalkingAnimation,
};

/// Builds a `WalkingAnimation` configured for the hag sprite sheet (4×4 frames).
pub fn hag_walking_animation(rng: &mut impl Rng) -> WalkingAnimation {
    let mut anim = WalkingAnimation::new_staggered(rng);
    anim.columns = HAG_SHEET_COLUMNS;
    anim.frame_uv = HAG_FRAME_UV;
    anim.direction_rows = HAG_DIRECTION_ROWS;
    anim
}

/// Builds a `CombatAnimation` for a hag using one of her sprite sheets
/// (attack or cast). All hag sheets share the same frame size; only the
/// row order, column count, and combat texture differ.
fn hag_combat_animation(
    hag_assets: &HagAssets,
    columns: usize,
    direction_rows: [usize; 4],
    combat_texture: Handle<Image>,
) -> CombatAnimation {
    CombatAnimation {
        current_frame: 0,
        elapsed: 0.0,
        columns,
        frame_uv: HAG_FRAME_UV,
        direction_rows,
        combat_texture,
        walking_texture: hag_assets.walking_texture.clone(),
        started: false,
    }
}

pub fn hag_attack_animation(hag_assets: &HagAssets) -> CombatAnimation {
    hag_combat_animation(
        hag_assets,
        HAG_ATTACK_COLUMNS,
        HAG_DIRECTION_ROWS,
        hag_assets.attacking_texture.clone(),
    )
}

pub fn hag_casting_animation(hag_assets: &HagAssets) -> CombatAnimation {
    hag_combat_animation(
        hag_assets,
        HAG_CASTING_COLUMNS,
        HAG_CASTING_DIRECTION_ROWS,
        hag_assets.casting_texture.clone(),
    )
}

/// Pins a hag's material to a specific frame of the attacking sprite sheet
/// for her current facing direction. Used to lock Josephina's leap pose.
pub fn set_hag_attack_pose_frame(
    materials: &mut Assets<StandardMaterial>,
    material_handle: &MeshMaterial3d<StandardMaterial>,
    hag_assets: &HagAssets,
    facing: FacingDirection,
    frame_idx: usize,
) {
    if let Some(mat) = materials.get_mut(material_handle) {
        let row = HAG_DIRECTION_ROWS[facing as usize] as f32;
        let offset = Vec2::new(frame_idx as f32 * HAG_FRAME_UV.x, row * HAG_FRAME_UV.y);
        mat.base_color_texture = Some(hag_assets.attacking_texture.clone());
        mat.uv_transform =
            bevy::math::Affine2::from_scale_angle_translation(HAG_FRAME_UV, 0.0, offset);
    }
}

/// Restores a hag's material to the walking sheet, frame 0 in her facing direction.
pub fn restore_hag_walking_pose(
    materials: &mut Assets<StandardMaterial>,
    material_handle: &MeshMaterial3d<StandardMaterial>,
    hag_assets: &HagAssets,
    facing: FacingDirection,
) {
    if let Some(mat) = materials.get_mut(material_handle) {
        let row = HAG_DIRECTION_ROWS[facing as usize] as f32;
        let offset = Vec2::new(0.0, row * HAG_FRAME_UV.y);
        mat.base_color_texture = Some(hag_assets.walking_texture.clone());
        mat.uv_transform =
            bevy::math::Affine2::from_scale_angle_translation(HAG_FRAME_UV, 0.0, offset);
    }
}

/// Builds a `PulsingAnimation` for an eye sprite (4-frame in-place loop).
pub fn eye_pulsing_animation() -> PulsingAnimation {
    PulsingAnimation::new(EYE_SHEET_COLUMNS, EYE_FRAME_UV, EYE_PULSE_FRAME_DURATION)
}
