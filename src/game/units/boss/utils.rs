use bevy::prelude::*;

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
