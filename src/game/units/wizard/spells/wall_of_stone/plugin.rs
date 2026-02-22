use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::WallOfStone;
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

/// Plugin that handles the Wall of Stone spell.
pub struct WallOfStonePlugin;

impl Plugin for WallOfStonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_wall_of_stone_cancel.run_if(spell_is_primed(Spell::WallOfStone)),
                // Local wizard casting (mouse input)
                systems::handle_wall_of_stone_casting
                    .run_if(spell_is_primed(Spell::WallOfStone))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_wall_of_stone_casting_guest
                    .run_if(guest_spell_is_primed(Spell::WallOfStone))
                    .run_if(guest_input_or_wizard_casting),
                (
                    systems::tick_wall_lifetime,
                    systems::animate_sinking_walls,
                    systems::cleanup_expired_walls,
                )
                    .chain()
                    .run_if(any_exist::<WallOfStone>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
