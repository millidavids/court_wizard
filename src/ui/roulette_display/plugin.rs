use bevy::prelude::*;

use crate::game::run_conditions;
use crate::game::run_conditions::is_local_wizard_active;
use crate::state::{InGameState, MultiplayerGameState};

use super::components::SelectedSpellFadeTimer;
use super::systems;

/// Plugin managing the roulette display UI for the Randomancer archetype.
pub struct RouletteDisplayPlugin;

impl Plugin for RouletteDisplayPlugin {
    fn build(&self, app: &mut App) {
        // SP spawn + cleanup (cleanup prevents duplicates on pause/unpause)
        app.add_systems(
            OnEnter(InGameState::Running),
            systems::spawn_roulette_display.run_if(run_conditions::is_randomancer),
        )
        .add_systems(
            OnExit(InGameState::Running),
            crate::ui::systems::cleanup_screen::<super::components::RouletteDisplayRoot>,
        )
        // MP spawn
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            systems::spawn_roulette_display.run_if(run_conditions::is_randomancer),
        )
        .add_systems(
            Update,
            (
                systems::animate_wheel_spin,
                systems::update_roulette_display,
                // Runs after `update_roulette_display` so it has the last
                // word on the Idle-phase prompt text / font.
                systems::adapt_prompt_to_input_device.after(systems::update_roulette_display),
                systems::update_selected_spell_fade
                    .run_if(any_with_component::<SelectedSpellFadeTimer>),
            )
                .run_if(is_local_wizard_active)
                .run_if(run_conditions::is_randomancer),
        );
    }
}
