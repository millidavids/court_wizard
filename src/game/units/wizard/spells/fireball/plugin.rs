use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::state::InGameState;

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
                systems::handle_fireball_casting
                    .run_if(spell_is_primed(Spell::Fireball))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
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
