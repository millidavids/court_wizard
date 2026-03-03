use bevy::prelude::*;
use bevy::prelude::UiMaterialPlugin;

use crate::state::{AppState, MetaGameState};
use crate::ui::plugin::ButtonActionSet;

use super::components::{GraphViewAnimation, GraphViewState, SelectedStudySpell};
use super::materials::{RadialProgressMaterial, StarSkyMaterial};
use super::systems::*;

pub struct WizardTowerPlugin;

impl Plugin for WizardTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<RadialProgressMaterial>::default())
            .add_plugins(UiMaterialPlugin::<StarSkyMaterial>::default())
            // Top-level cleanup when leaving MetaGame entirely
            .add_systems(OnExit(AppState::MetaGame), (crate::ui::systems::cleanup_screen::<super::components::OnWizardTowerScreen>, cleanup_wizard_tower_resources))
            // WizardTower substate (hub screen)
            .add_systems(OnEnter(MetaGameState::WizardTower), setup_wizard_tower_main)
            .add_systems(OnExit(MetaGameState::WizardTower), crate::ui::systems::cleanup_screen::<super::components::OnMainScreen>)
            .add_systems(
                Update,
                handle_main_button_actions
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // Study substate
            .add_systems(OnEnter(MetaGameState::Study), setup_study_screen)
            .add_systems(OnExit(MetaGameState::Study), (crate::ui::systems::cleanup_screen::<super::components::OnStudyScreen>, cleanup_study_resources))
            .add_systems(
                Update,
                (
                    handle_study_button_actions.in_set(ButtonActionSet),
                    handle_graph_node_clicks.in_set(ButtonActionSet),
                )
                    .run_if(in_state(MetaGameState::Study)),
            )
            .add_systems(
                Update,
                update_star_sky_time.run_if(in_state(MetaGameState::Study)),
            )
            .add_systems(
                Update,
                (
                    handle_graph_pan,
                    handle_graph_zoom,
                    animate_graph_view
                        .run_if(resource_exists::<GraphViewAnimation>),
                    update_graph_node_positions
                        .run_if(resource_exists::<GraphViewState>),
                    update_graph_edge_positions
                        .run_if(resource_exists::<GraphViewState>),
                    update_graph_node_borders
                        .run_if(resource_exists::<SelectedStudySpell>.and(resource_changed::<SelectedStudySpell>)),
                    handle_detail_slider_interaction,
                    update_detail_sliders,
                    update_study_detail_panel,
                    update_allocation_text,
                    update_pending_insight_display,
                    handle_talent_card_clicks,
                    update_talent_hover_description,
                    clear_talent_hover_description,
                )
                    .run_if(in_state(MetaGameState::Study)),
            );
    }
}
