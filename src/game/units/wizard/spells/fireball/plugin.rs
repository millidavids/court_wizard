use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles fireball spell casting and behavior.
///
/// Registers systems for:
/// - Casting fireballs with mouse button and cast time
/// - Fireball projectile movement
/// - Collision detection (units and ground)
/// - Explosion animation and damage
/// - Cleanup for finished explosions
pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_fireball_casting,
                systems::move_fireballs,
                systems::check_fireball_collisions,
                systems::despawn_distant_fireballs,
                systems::update_explosions,
                systems::apply_explosion_damage,
                systems::cleanup_finished_explosions,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
