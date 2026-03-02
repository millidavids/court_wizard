//! Wizard select screen plugin.

use bevy::prelude::*;

use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::escape_to_landing;

use super::systems;

/// Plugin that manages the wizard select screen UI.
#[derive(Default)]
pub struct WizardSelectPlugin;

impl Plugin for WizardSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::WizardSelect), systems::setup)
            .add_systems(OnExit(MenuState::WizardSelect), (crate::ui::systems::cleanup_screen::<super::components::OnWizardSelectScreen>, systems::cleanup_wizard_select_resources))
            .add_systems(
                Update,
                systems::button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::WizardSelect)),
            )
            .add_systems(
                Update,
                escape_to_landing.run_if(in_state(MenuState::WizardSelect)),
            );
    }
}
