use bevy::prelude::*;

/// Eye-pulsing sprite sheet — shared by hag invulnerability/ability eyes and
/// Ray's 5 boss eyes. 4 frames in a single 256×64 row.
pub(in crate::game) const EYE_SHEET_WIDTH: f32 = 256.0;
pub(in crate::game) const EYE_SHEET_HEIGHT: f32 = 64.0;
pub(in crate::game) const EYE_SHEET_COLUMNS: usize = 4;
pub(in crate::game) const EYE_PULSE_FRAME_DURATION: f32 = 0.18;
/// UV size of one eye frame (frames are square, 64×64 within a 256×64 sheet).
pub(in crate::game) const EYE_FRAME_UV: Vec2 = Vec2::new(EYE_SHEET_HEIGHT / EYE_SHEET_WIDTH, 1.0);

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
