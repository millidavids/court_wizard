//! Wizard select screen plugin.

use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that manages the wizard select screen UI.
#[derive(Default)]
pub struct WizardSelectPlugin;

impl Plugin for WizardSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::WizardSelect), systems::setup)
            .add_systems(OnExit(MenuState::WizardSelect), systems::cleanup)
            .add_systems(
                Update,
                systems::button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::WizardSelect)),
            )
            .add_systems(
                Update,
                systems::keyboard_input.run_if(in_state(MenuState::WizardSelect)),
            );
    }
}
