use bevy::prelude::*;

use crate::game::run_conditions;
use crate::game::run_conditions::is_local_wizard_active;
use crate::state::{InGameState, MultiplayerGameState};

use super::messages::*;
use super::resources::RouletteState;
use super::systems;

/// Plugin managing the roulette wheel system for the Randomancer archetype.
pub struct RoulettePlugin;

impl Plugin for RoulettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RouletteState>()
            .add_message::<RouletteSpinMessage>()
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                systems::reset_roulette_state.run_if(run_conditions::is_randomancer),
            )
            // Reset the wheel on the multiplayer score screen too (the SP
            // `InGameState::ScoreScreen` reset never fires in MP).
            .add_systems(
                OnEnter(MultiplayerGameState::ScoreScreen),
                systems::reset_roulette_state.run_if(run_conditions::is_randomancer),
            )
            // The spin state machine only mutates the local `RouletteState` and
            // emits `PrimeSpellMessage` — no simulation authority needed. Gate on
            // `is_local_wizard_active` (both peers) so the GUEST Randomancer's
            // wheel actually spins; under `is_gameplay_running` (host-only) the
            // guest's spin trigger is collected but never advanced.
            .add_systems(
                Update,
                (
                    systems::handle_spin_trigger,
                    systems::update_spin,
                    systems::reset_after_cast,
                )
                    .chain()
                    .run_if(is_local_wizard_active)
                    .run_if(run_conditions::is_randomancer),
            );
    }
}
