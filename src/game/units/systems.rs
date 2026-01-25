use bevy::prelude::*;

use super::components::TemporaryHitPoints;

/// Updates all temporary hit points timers and removes expired components.
///
/// This system runs each frame to:
/// - Decrement time_remaining on all TemporaryHitPoints components
/// - Remove components that have expired (time <= 0 or amount <= 0)
pub fn update_temporary_hit_points(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TemporaryHitPoints)>,
) {
    let delta = time.delta_secs();

    for (entity, mut temp_hp) in query.iter_mut() {
        if temp_hp.update(delta) {
            // Temp HP has expired, remove the component
            commands.entity(entity).remove::<TemporaryHitPoints>();
        }
    }
}
