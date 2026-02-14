use bevy::prelude::*;

use crate::state::InGameState;
use crate::ui::plugin::ButtonActionSet;

use super::components::NewlyUnlockedIngredient;
use super::systems::*;

pub struct WizardTowerPlugin;

impl Plugin for WizardTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewlyUnlockedIngredient>()
            .add_systems(
                OnEnter(InGameState::WizardTower),
                (try_unlock_random_ingredient, setup_wizard_tower_screen).chain(),
            )
            .add_systems(
                OnExit(InGameState::WizardTower),
                cleanup_wizard_tower_screen,
            )
            .add_systems(
                Update,
                handle_button_actions
                    .in_set(ButtonActionSet)
                    .run_if(in_state(InGameState::WizardTower)),
            );
    }
}
