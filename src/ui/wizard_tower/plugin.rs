use bevy::prelude::*;
use bevy::prelude::UiMaterialPlugin;

use crate::state::{AppState, MetaGameState};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::handle_scroll;

use super::components::{
    GraphViewAnimation, GraphViewState, SelectedInsightBonus, SelectedStudySpell,
    SelectedTimeTravelLevel, TimeTravelSection,
};
use super::layout::{
    RightPanelView, WizardTowerTab,
};
use super::materials::{ConcentricRingsMaterial, RadialProgressMaterial, StarSkyMaterial};

pub struct WizardTowerPlugin;

impl Plugin for WizardTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<RadialProgressMaterial>::default())
            .add_plugins(UiMaterialPlugin::<ConcentricRingsMaterial>::default())
            .add_plugins(UiMaterialPlugin::<StarSkyMaterial>::default())
            // ----- Top-level cleanup when leaving MetaGame entirely -----
            .add_systems(
                OnExit(AppState::MetaGame),
                (
                    crate::ui::systems::cleanup_screen::<super::components::OnWizardTowerScreen>,
                    super::layout::cleanup_wizard_tower_tab_resources,
                    super::study_tab::cleanup_study_resources,
                    super::roguelite_tab::cleanup_roguelite_tab_resources,
                ),
            )
            // ----- WizardTower substate: tabbed hub -----
            .add_systems(
                OnEnter(MetaGameState::WizardTower),
                super::layout::setup_wizard_tower_layout,
            )
            .add_systems(
                OnExit(MetaGameState::WizardTower),
                crate::ui::systems::cleanup_screen::<super::components::OnMainScreen>,
            )
            // ----- Tab switching and layout systems -----
            .add_systems(
                Update,
                (
                    super::layout::handle_tab_click.in_set(ButtonActionSet),
                    super::layout::handle_back_button.in_set(ButtonActionSet),
                    super::layout::escape_to_main_menu,
                    super::layout::rebuild_panels_on_tab_change.run_if(
                        resource_exists::<WizardTowerTab>.and(
                            resource_changed::<WizardTowerTab>
                                .or(resource_changed::<RightPanelView>)
                                .or(resource_removed::<crate::game::game_mode::components::RogueliteRunState>),
                        ),
                    ),
                    super::layout::update_tab_active_state
                        .run_if(resource_exists::<WizardTowerTab>),
                    handle_scroll::<super::layout::WizardTowerLeftPanel>,
                    handle_scroll::<super::layout::WizardTowerRightPanel>,
                )
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // ----- Wizard card grid systems -----
            .add_systems(
                Update,
                (
                    super::wizard_cards::handle_wizard_card_actions
                        .in_set(ButtonActionSet),
                    super::wizard_cards::animate_card_expand,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(resource_exists::<super::wizard_cards::ExpandedWizard>)
                    .run_if(resource_exists::<RightPanelView>.and(
                        |view: Res<RightPanelView>| *view == RightPanelView::WizardSelect,
                    )),
            )
            // ----- Study tab systems -----
            .add_systems(
                Update,
                (
                    super::study_tab::handle_study_button_actions.in_set(ButtonActionSet),
                    super::study_tab::handle_graph_node_clicks.in_set(ButtonActionSet),
                    super::study_tab::handle_talent_card_clicks.in_set(ButtonActionSet),
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<SelectedStudySpell>),
            )
            .add_systems(
                Update,
                super::study_tab::update_star_sky_time
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<SelectedStudySpell>),
            )
            .add_systems(
                Update,
                (
                    super::study_tab::handle_graph_pan,
                    super::study_tab::handle_graph_zoom,
                    super::study_tab::animate_graph_view
                        .run_if(resource_exists::<GraphViewAnimation>),
                    super::study_tab::update_graph_node_positions
                        .run_if(resource_exists::<GraphViewState>),
                    super::study_tab::update_graph_edge_positions
                        .run_if(resource_exists::<GraphViewState>),
                    super::study_tab::update_insight_node_positions
                        .run_if(resource_exists::<GraphViewState>),
                    super::study_tab::update_insight_edge_positions
                        .run_if(resource_exists::<GraphViewState>),
                    super::study_tab::update_graph_node_borders.run_if(
                        resource_exists::<SelectedStudySpell>
                            .and(resource_changed::<SelectedStudySpell>),
                    ),
                    super::study_tab::update_insight_node_borders.run_if(
                        resource_exists::<SelectedInsightBonus>
                            .and(resource_changed::<SelectedInsightBonus>),
                    ),
                    super::study_tab::handle_detail_slider_interaction,
                    super::study_tab::handle_insight_bonus_slider_interaction,
                    super::study_tab::update_detail_sliders,
                    super::study_tab::update_insight_bonus_sliders,
                    super::study_tab::update_study_detail_panel,
                    super::study_tab::update_insight_detail_panel
                        .after(super::study_tab::update_study_detail_panel),
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<SelectedStudySpell>),
            )
            .add_systems(
                Update,
                (
                    super::study_tab::update_allocation_text,
                    super::study_tab::update_insight_bonus_allocation_text,
                    super::study_tab::update_insight_bonus_rings,
                    super::study_tab::update_graph_node_label_scale
                        .run_if(resource_exists::<GraphViewState>),
                    super::study_tab::update_pending_insight_display,
                    super::study_tab::update_talent_hover_description,
                    super::study_tab::clear_talent_hover_description,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<SelectedStudySpell>),
            )
            // ----- Roguelite tab systems -----
            // Action handler runs whenever the roguelite tab is active (handles
            // both "no run" and "active run" views)
            .add_systems(
                Update,
                super::roguelite_tab::handle_roguelite_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(roguelite_tab_active),
            )
            // Modifier UI systems only run when the modifier resources exist
            // (i.e. the "no active run" view is showing)
            .add_systems(
                Update,
                (
                    super::roguelite_tab::slider_button_action.in_set(ButtonActionSet),
                    super::roguelite_tab::toggle_expand_action.in_set(ButtonActionSet),
                    super::roguelite_tab::toggle_row_action.in_set(ButtonActionSet),
                    super::roguelite_tab::handle_unlock_confirmation.in_set(ButtonActionSet),
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(roguelite_tab_active)
                    .run_if(resource_exists::<super::roguelite_tab::SeedInputState>),
            )
            .add_systems(
                Update,
                (
                    super::roguelite_tab::slider_interaction,
                    super::roguelite_tab::update_sliders,
                    super::roguelite_tab::update_slider_text,
                    super::roguelite_tab::update_run_summary,
                    super::roguelite_tab::seed_input_click,
                    super::roguelite_tab::seed_input_keyboard,
                    handle_scroll::<super::roguelite_tab::RogueliteScrollableContent>,
                    handle_scroll::<super::roguelite_tab::RogueliteScrollableLeft>,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(roguelite_tab_active)
                    .run_if(resource_exists::<super::roguelite_tab::SeedInputState>),
            )
            // ----- Endless tab systems -----
            .add_systems(
                Update,
                (
                    super::endless_tab::handle_endless_actions.in_set(ButtonActionSet),
                    super::endless_tab::handle_time_travel_level_clicks
                        .in_set(ButtonActionSet)
                        .run_if(resource_exists::<SelectedTimeTravelLevel>),
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(endless_tab_active),
            )
            .add_systems(
                Update,
                (
                    super::endless_tab::handle_time_travel_level_hover,
                    handle_scroll::<TimeTravelSection>,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(endless_tab_active),
            );
    }
}

// ----- Run condition helpers -----

fn study_tab_active(tab: Option<Res<WizardTowerTab>>) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Study)
}

fn roguelite_tab_active(tab: Option<Res<WizardTowerTab>>, view: Option<Res<RightPanelView>>) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Roguelite)
        && view.is_some_and(|v| *v == RightPanelView::TabContent)
}

fn endless_tab_active(tab: Option<Res<WizardTowerTab>>, view: Option<Res<RightPanelView>>) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Endless)
        && view.is_some_and(|v| *v == RightPanelView::TabContent)
}
