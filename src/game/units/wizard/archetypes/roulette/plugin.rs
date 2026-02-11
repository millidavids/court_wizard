use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::InGameState;

use super::messages::*;
use super::resources::RouletteState;
use super::systems;

/// Plugin managing the roulette wheel system for the Randomancer archetype.
pub struct RoulettePlugin;

impl Plugin for RoulettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RouletteState>()
            .add_message::<RouletteSpinMessage>()
            .add_message::<RouletteSelectedMessage>()
            .add_systems(
                OnEnter(InGameState::GameOver),
                systems::reset_roulette_state.run_if(run_conditions::is_randomancer),
            )
            .add_systems(
                Update,
                (
                    systems::handle_spin_trigger,
                    systems::update_spin,
                    systems::reset_after_cast,
                )
                    .chain()
                    .run_if(in_state(InGameState::Running))
                    .run_if(run_conditions::is_randomancer),
            );
    }
}
