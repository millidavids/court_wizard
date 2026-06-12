use bevy::prelude::*;

/// Marker inserted alongside each forwarded status component so the
/// forwarder doesn't re-ship the same effect every frame for as long as the
/// component lives. Component-typed so adding a new status doesn't risk
/// colliding with an existing marker. The companion system
/// `cleanup_forwarded_marker::<T>` removes this marker the frame after `T`
/// is removed from the ghost, so a re-cast of the same status on the same
/// ghost is re-forwarded instead of silently dropped.
#[derive(Component)]
pub struct StatusEffectForwarded<T: Component> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Component> Default for StatusEffectForwarded<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Removes `StatusEffectForwarded<T>` from any ghost entity whose underlying
/// status component `T` was just removed (expired, dispelled, etc.). Run as
/// one instance per concrete `T` so each marker is paired with the exact
/// component that drives it.
pub fn cleanup_forwarded_marker<T: Component>(
    mut commands: Commands,
    forwarded: Query<Entity, (With<StatusEffectForwarded<T>>, Without<T>)>,
) {
    for entity in &forwarded {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.remove::<StatusEffectForwarded<T>>();
        }
    }
}
