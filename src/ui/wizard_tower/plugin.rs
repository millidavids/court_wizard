use bevy::prelude::*;

use crate::state::{AppState, MetaGameState};
use crate::ui::plugin::ButtonActionSet;

use super::systems::*;

pub struct WizardTowerPlugin;

impl Plugin for WizardTowerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Top-level cleanup when leaving MetaGame entirely
            .add_systems(OnExit(AppState::MetaGame), cleanup_wizard_tower_screen)
            // WizardTower substate (hub screen)
            .add_systems(OnEnter(MetaGameState::WizardTower), setup_wizard_tower_main)
            .add_systems(OnExit(MetaGameState::WizardTower), cleanup_main_screen)
            .add_systems(
                Update,
                handle_main_button_actions
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // Study substate
            .add_systems(OnEnter(MetaGameState::Study), setup_study_screen)
            .add_systems(OnExit(MetaGameState::Study), cleanup_study_screen)
            .add_systems(
                Update,
                handle_study_button_actions
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MetaGameState::Study)),
            )
            .add_systems(
                Update,
                (
                    handle_allocation_slider_interaction,
                    handle_study_scroll,
                    update_allocation_sliders,
                    update_allocation_text,
                    update_pending_insight_display,
                )
                    .run_if(in_state(MetaGameState::Study)),
            );
    }
}
