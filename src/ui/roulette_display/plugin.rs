use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::{AppState, InGameState};

use super::components::SelectedSpellFadeTimer;
use super::systems;

/// Plugin managing the roulette display UI for the Randomancer archetype.
pub struct RouletteDisplayPlugin;

impl Plugin for RouletteDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            systems::spawn_roulette_display.run_if(run_conditions::is_randomancer),
        )
        .add_systems(
            OnEnter(InGameState::Running),
            systems::spawn_roulette_display
                .run_if(run_conditions::coming_from_game_over)
                .run_if(run_conditions::is_randomancer),
        )
        .add_systems(
            Update,
            (
                systems::animate_wheel_spin,
                systems::update_roulette_display,
                systems::update_selected_spell_fade
                    .run_if(any_with_component::<SelectedSpellFadeTimer>),
            )
                .run_if(in_state(InGameState::Running))
                .run_if(run_conditions::is_randomancer),
        );
    }
}
