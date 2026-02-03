//! Unified movement system that applies velocity to position.
//!
//! This system runs AFTER all unit-specific movement calculations and
//! external forces (like gravity) have been applied to velocity.

use bevy::prelude::*;

use crate::game::components::{Acceleration, Velocity};

/// Applies velocity to position for all units.
///
/// This system should run AFTER:
/// - Unit movement calculations (infantry_movement, archer_movement, king_movement)
/// - External force applications (black hole gravity, etc.)
///
/// It integrates all accumulated acceleration into velocity, then applies velocity to position,
/// and finally resets acceleration for the next frame.
pub fn apply_unit_movement(
    time: Res<Time>,
    mut units: Query<(&mut Transform, &mut Velocity, &mut Acceleration)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut velocity, mut acceleration) in units.iter_mut() {
        // FIRST: Integrate all accumulated acceleration into velocity
        // This includes both normal movement acceleration AND external forces (like gravity)
        velocity.x += acceleration.x * delta;
        velocity.z += acceleration.z * delta;

        // THEN: Apply velocity to position (only XZ plane - Y stays fixed at spawn height)
        transform.translation.x += velocity.x * delta;
        transform.translation.z += velocity.z * delta;

        // FINALLY: Reset acceleration for next frame
        acceleration.reset();
    }
}
