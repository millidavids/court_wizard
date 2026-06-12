use bevy::prelude::*;

use crate::game::units::components::{ANIMATION_MOVE_THRESHOLD_SQ, FacingDirection};

/// Eye-pulsing sprite sheet — shared by hag invulnerability/ability eyes and
/// Ray's 5 boss eyes. 4 frames in a single 256×64 row.
pub(in crate::game) const EYE_SHEET_COLUMNS: usize = 4;
pub(in crate::game) const EYE_PULSE_FRAME_DURATION: f32 = 0.18;
/// UV size of one eye frame (frames are square, 64×64 within a 256×64 sheet).
pub(in crate::game) const EYE_FRAME_UV: Vec2 = Vec2::new(64.0 / 256.0, 1.0);

/// Lay a rectangle flat on the XZ plane with its long axis aligned to `direction`.
/// Shared by boss telegraph/indicator systems.
pub(in crate::game) fn indicator_rotation(direction: Vec3) -> Quat {
    let angle = (-direction.x).atan2(-direction.z);
    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_z(angle)
}

/// Despawn a slice of indicator entities.
pub(in crate::game) fn despawn_indicators(commands: &mut Commands, entities: &[Entity]) {
    for &entity in entities {
        commands.entity(entity).try_despawn();
    }
}

/// Pulse frequency for boss telegraph indicators (Hz).
pub(in crate::game) const TELEGRAPH_PULSE_FREQUENCY: f32 = 2.5;
/// Maximum emissive intensity for telegraph red channel.
pub(in crate::game) const TELEGRAPH_EMISSIVE_MAX: f32 = 8.0;

/// Animate a telegraph indicator material's emissive glow + base-color alpha.
///
/// Both ogre and dark_mage telegraphs use this exact pattern: a sinusoidal pulse
/// modulates the emissive while the base-color alpha and intensity ramp linearly
/// with `progress` (0.0 at start, 1.0 at impact).
///
/// `base_red` is the only difference between bosses (ogre uses 0.6, dark_mage 0.8).
pub(in crate::game) fn animate_telegraph_material(
    mat: &mut StandardMaterial,
    elapsed: f32,
    progress: f32,
    base_red: f32,
) {
    let pulse = (elapsed * TELEGRAPH_PULSE_FREQUENCY * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let intensity = progress * TELEGRAPH_EMISSIVE_MAX * (0.6 + 0.4 * pulse);
    mat.emissive = bevy::color::LinearRgba::new(intensity, intensity * 0.08, intensity * 0.02, 1.0);
    let alpha = 0.1 + progress * 0.4;
    mat.base_color = Color::srgba(base_red, base_red * 0.125, base_red * 0.025, alpha);
}

/// Two-direction camera-relative facing for floating bosses (lich, dark mage).
/// Returns `Some(Back)` when moving in a >2*acos(threshold)° arc directly away
/// from the camera, else `Some(Forward)`. Returns `None` when the boss is
/// effectively stationary so the caller can leave facing unchanged.
pub(in crate::game) fn back_facing_for_velocity(
    velocity_xz: Vec3,
    cam_forward_xz: Vec3,
    threshold: f32,
) -> Option<FacingDirection> {
    let speed_sq = velocity_xz.length_squared();
    if speed_sq < ANIMATION_MOVE_THRESHOLD_SQ {
        return None;
    }
    let speed = speed_sq.sqrt();
    let forward_dot = velocity_xz.dot(cam_forward_xz);
    Some(if forward_dot > threshold * speed {
        FacingDirection::Back
    } else {
        FacingDirection::Forward
    })
}

/// Sinusoidal hover bob, sized in world units. Shared by the lich and dark
/// mage float systems.
pub(in crate::game) fn sinusoidal_bob(amplitude: f32, frequency_hz: f32, elapsed_secs: f32) -> f32 {
    amplitude * (elapsed_secs * frequency_hz * std::f32::consts::TAU).sin()
}

/// Camera viewport visibility test. Returns true when `world_pos` projects
/// inside the `[-margin, margin]` NDC box. Margin in (0, 1] — 1.0 is the
/// hard screen edge, lower values inset for safety.
pub(in crate::game) fn is_on_screen(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_pos: Vec3,
    margin: f32,
) -> bool {
    let Some(ndc) = camera.world_to_ndc(camera_transform, world_pos) else {
        return false;
    };
    ndc.x.abs() <= margin && ndc.y.abs() <= margin
}
