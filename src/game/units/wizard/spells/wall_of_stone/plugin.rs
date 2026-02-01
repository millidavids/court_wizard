use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::state::InGameState;

/// Plugin that handles the Wall of Stone spell.
pub struct WallOfStonePlugin;

impl Plugin for WallOfStonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_wall_of_stone_cancel.run_if(spell_is_primed(Spell::WallOfStone)),
                systems::handle_wall_of_stone_casting
                    .run_if(spell_is_primed(Spell::WallOfStone))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::tick_wall_lifetime,
                systems::animate_sinking_walls,
                systems::cleanup_expired_walls,
            )
                .run_if(in_state(InGameState::Running)),
        );
    }
}
