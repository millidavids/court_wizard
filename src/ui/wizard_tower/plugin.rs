use bevy::prelude::*;

use crate::state::InGameState;
use crate::ui::plugin::ButtonActionSet;

use super::systems::*;

pub struct WizardTowerPlugin;

impl Plugin for WizardTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::WizardTower), setup_wizard_tower_screen)
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
