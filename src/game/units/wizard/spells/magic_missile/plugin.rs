use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::MagicMissile;
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

/// Plugin that handles magic missile spell casting and behavior.
///
/// Registers systems for:
/// - Casting magic missiles with mouse button and cast time
/// - Magic missile homing movement with wobble
/// - Collision detection and damage
/// - Cleanup for distant missiles
pub struct MagicMissilePlugin;

impl Plugin for MagicMissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_magic_missile_casting
                    .run_if(spell_is_primed(Spell::MagicMissile))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::move_magic_missiles,
                    systems::check_magic_missile_collisions,
                    systems::despawn_distant_magic_missiles,
                )
                    .chain()
                    .run_if(any_exist::<MagicMissile>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
