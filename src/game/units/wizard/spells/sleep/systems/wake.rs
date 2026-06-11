use crate::game::units::components::{MovementSpeed, Sleepwalking, TargetingVelocity};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use bevy::prelude::*;

/// Dreamwalker: sleepwalking units walk away from the local wizard's castle (back toward spawn).
/// Overrides their targeting velocity so they move in the opposite direction of `LocalSpellOrigin`,
/// which resolves to the host's wizard in SP/host and the guest's wizard in MP guest.
pub fn update_sleepwalkers(
    mut query: Query<(
        &Transform,
        &Sleepwalking,
        &mut TargetingVelocity,
        &MovementSpeed,
    )>,
    local_origin: Res<LocalSpellOrigin>,
) {
    for (transform, sleepwalking, mut targeting, movement_speed) in query.iter_mut() {
        let away_dir = transform.translation - local_origin.0;
        let horizontal = Vec3::new(away_dir.x, 0.0, away_dir.z);
        let length = horizontal.length();

        if length > 0.001 {
            let normalized = horizontal / length;
            targeting.velocity = normalized * movement_speed.0 * sleepwalking.speed_mult;
            // Set a large distance so flocking/flow fields don't override
            targeting.distance_to_target = 1000.0;
        }
    }
}
