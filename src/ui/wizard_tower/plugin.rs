use bevy::prelude::UiMaterialPlugin;
use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;
use crate::state::{AppState, MetaGameState};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::handle_scroll;

use super::components::{
    GraphViewAnimation, GraphViewState, SelectedInsightBonus, SelectedStudySpell,
    SelectedTimeTravelLevel, TimeTravelSection,
};
use super::layout::{RightPanelView, WizardTowerTab};
use super::materials::{
    ArcaneRuneMaterial, ConcentricRingsMaterial, RadialProgressMaterial, StarSkyMaterial,
};
use super::multiplayer_tab::{MultiplayerLobby, MultiplayerTabPlugin};

pub struct WizardTowerPlugin;

impl Plugin for WizardTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MultiplayerTabPlugin)
            .add_plugins(UiMaterialPlugin::<ArcaneRuneMaterial>::default())
            .add_plugins(UiMaterialPlugin::<RadialProgressMaterial>::default())
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
            // ----- Arcane rune background animation -----
            .add_systems(
                Update,
                (
                    super::layout::update_arcane_rune_time,
                    super::layout::update_arcane_rune_text,
                    super::layout::rebuild_rune_on_spell_unlock,
                )
                    .run_if(in_state(MetaGameState::WizardTower)),
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
                                .or(resource_removed::<
                                    crate::game::game_mode::components::RogueliteRunState,
                                >),
                        ),
                    ),
                    // Tab switches are handled by `rebuild_panels_on_tab_change`;
                    // this only handles mid-tab lobby/connection changes, so it
                    // must NOT also fire on `WizardTowerTab` change or the panels
                    // would be built twice in one frame.
                    rebuild_multiplayer_on_lobby_change
                        .after(super::layout::rebuild_panels_on_tab_change)
                        .run_if(
                            resource_exists::<WizardTowerTab>
                                .and(multiplayer_tab_active)
                                .and(
                                    resource_changed::<MultiplayerLobby>
                                        .or(resource_changed::<NetworkConnection>),
                                ),
                        ),
                    super::layout::update_tab_active_state
                        .run_if(resource_exists::<WizardTowerTab>),
                    handle_scroll::<super::layout::WizardTowerLeftPanel>,
                    handle_scroll::<super::layout::WizardTowerRightPanel>,
                    handle_scroll::<super::wizard_cards::WizardCardScrollContainer>,
                )
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // ----- Wizard card grid systems -----
            .add_systems(
                Update,
                (
                    super::wizard_cards::handle_wizard_card_actions.in_set(ButtonActionSet),
                    super::wizard_cards::animate_card_expand,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(
                        resource_exists::<RightPanelView>
                            .and(|view: Res<RightPanelView>| *view == RightPanelView::WizardSelect)
                            .and(resource_exists::<super::wizard_cards::SelectedWizard>),
                    ),
            )
            // ----- Study tab systems -----
            .add_systems(
                Update,
                (
                    super::study_tab::handle_study_button_actions.in_set(ButtonActionSet),
                    super::study_tab::handle_graph_node_clicks.in_set(ButtonActionSet),
                    super::study_tab::handle_talent_card_clicks.in_set(ButtonActionSet),
                    super::study_tab::handle_alloc_adjust_buttons.in_set(ButtonActionSet),
                    // Writes MouseClicked while +/- is held, so it must NOT
                    // be in ButtonActionSet (that set is gated on a click
                    // already existing — chicken-and-egg).
                    super::study_tab::handle_alloc_adjust_buttons_hold,
                    super::study_tab::reset_previous_alloc_on_selection_change,
                    super::study_tab::toggle_focus_nav_on_study_selection,
                    // Must run AFTER escape_to_main_menu so escape sees the
                    // selection is still Some and skips its exit-to-menu path.
                    // study_back_to_cursor then clears the selection on the
                    // same back press.
                    super::study_tab::study_back_to_cursor
                        .after(super::layout::escape_to_main_menu),
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
                    super::study_tab::handle_graph_pan.run_if(not(resource_exists::<
                        crate::ui::tutorial::resources::ActiveTutorial,
                    >)),
                    super::study_tab::handle_graph_zoom.run_if(not(resource_exists::<
                        crate::ui::tutorial::resources::ActiveTutorial,
                    >)),
                    super::study_tab::animate_graph_view
                        .run_if(resource_exists::<GraphViewAnimation>),
                    // Position/layout systems only need to run when the view
                    // actually moves — gate on `resource_changed` so idle
                    // frames don't rewrite dozens of `Node` fields.
                    // `resource_changed` requires the resource to exist, so
                    // combine with `resource_exists`.
                    super::study_tab::update_graph_node_positions.run_if(
                        resource_exists::<GraphViewState>.and(resource_changed::<GraphViewState>),
                    ),
                    super::study_tab::update_graph_edge_positions.run_if(
                        resource_exists::<GraphViewState>.and(resource_changed::<GraphViewState>),
                    ),
                    super::study_tab::update_insight_node_positions.run_if(
                        resource_exists::<GraphViewState>.and(resource_changed::<GraphViewState>),
                    ),
                    super::study_tab::update_insight_edge_positions.run_if(
                        resource_exists::<GraphViewState>.and(resource_changed::<GraphViewState>),
                    ),
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
                    super::study_tab::update_graph_node_label_scale.run_if(
                        resource_exists::<GraphViewState>.and(resource_changed::<GraphViewState>),
                    ),
                    super::study_tab::update_pending_insight_display,
                    super::study_tab::update_talent_hover_description,
                    super::study_tab::clear_talent_hover_description,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<SelectedStudySpell>),
            )
            // ----- Study tab gamepad cursor -----
            // Spawn: runs while in WizardTower so the cursor appears when the
            // graph area is added on tab switch to Study.
            .add_systems(
                Update,
                super::study_tab::spawn_study_cursor_on_area_added
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // Cleanup: must run regardless of state so `FocusNavInhibit` is
            // cleared even when the area is despawned by leaving the wizard
            // tower entirely — otherwise focus navigation stays suppressed
            // on the main-menu landing screen.
            .add_systems(
                Update,
                super::study_tab::cleanup_study_cursor_on_area_removed,
            )
            // Cursor systems only run while we're actually in "spell-web cursor"
            // mode — i.e. `FocusNavInhibit` is active. As soon as a spell or
            // bonus is selected, `toggle_focus_nav_on_study_selection` removes
            // the inhibit and the cursor freezes so left-stick and A act on
            // the detail-panel focusables instead of re-clicking nodes.
            .add_systems(
                Update,
                (
                    super::study_tab::update_study_cursor,
                    super::study_tab::sync_study_cursor_visual
                        .after(super::study_tab::update_study_cursor),
                    super::study_tab::detect_study_cursor_hover
                        .after(super::study_tab::sync_study_cursor_visual),
                    super::study_tab::study_cursor_confirm
                        .after(super::study_tab::detect_study_cursor_hover),
                    super::study_tab::study_cursor_edge_scroll
                        .after(super::study_tab::update_study_cursor)
                        .run_if(resource_exists::<GraphViewState>),
                    super::study_tab::study_cursor_trigger_zoom
                        .run_if(resource_exists::<GraphViewState>),
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<super::study_tab::StudyCursorMode>)
                    .run_if(resource_exists::<crate::ui::focus::FocusNavInhibit>)
                    .run_if(not(resource_exists::<
                        crate::ui::tutorial::resources::ActiveTutorial,
                    >)),
            )
            // Reticle appearance (including show/hide based on mode) runs
            // regardless of inhibit state so the visual can vanish when the
            // detail panel opens.
            .add_systems(
                Update,
                super::study_tab::update_reticle_appearance
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(study_tab_active)
                    .run_if(resource_exists::<super::study_tab::StudyCursorMode>),
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

        // ----- Debug: F3 to toggle UI visibility -----
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            super::layout::toggle_debug_background.run_if(in_state(MetaGameState::WizardTower)),
        );

        // ----- Debug: hide/show debug-only widgets via the global F2 flag -----
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (
                crate::game::debug_ui::sync_marker_visibility::<
                    super::study_tab::panels::DebugInsightButton,
                >
                    .run_if(study_tab_active),
                crate::game::debug_ui::sync_marker_visibility::<
                    super::endless_tab::DebugLevelButtons,
                >
                    .run_if(endless_tab_active),
            )
                .run_if(in_state(MetaGameState::WizardTower)),
        );
    }
}

// ----- Run condition helpers -----

fn study_tab_active(tab: Option<Res<WizardTowerTab>>) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Study)
}

fn roguelite_tab_active(
    tab: Option<Res<WizardTowerTab>>,
    view: Option<Res<RightPanelView>>,
) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Roguelite)
        && view.is_some_and(|v| *v == RightPanelView::TabContent)
}

fn endless_tab_active(tab: Option<Res<WizardTowerTab>>, view: Option<Res<RightPanelView>>) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Endless)
        && view.is_some_and(|v| *v == RightPanelView::TabContent)
}

/// True only when the Multiplayer tab is showing its normal content — not the
/// shared wizard-card grid. Gates `rebuild_multiplayer_on_lobby_change` so a
/// lobby/connection change can't clobber the open grid.
fn multiplayer_tab_active(
    tab: Option<Res<WizardTowerTab>>,
    view: Option<Res<RightPanelView>>,
) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Multiplayer)
        && view.is_some_and(|v| *v == RightPanelView::TabContent)
}

// ---------------------------------------------------------------------------
// Multiplayer panel rebuild (driven by lobby/connection change)
// ---------------------------------------------------------------------------

/// Rebuilds the multiplayer left+right panels when `MultiplayerLobby` or
/// `NetworkConnection` changes while the Multiplayer tab is active.
///
/// This is a sibling to `rebuild_panels_on_tab_change` — that system fires
/// on tab-switch; this one fires when the lobby phase advances mid-tab.
#[allow(clippy::too_many_arguments)]
fn rebuild_multiplayer_on_lobby_change(
    mut commands: Commands,
    left_panel: Query<Entity, With<super::layout::WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<super::layout::WizardTowerRightPanel>>,
    lobby: Res<MultiplayerLobby>,
    connection: Res<NetworkConnection>,
) {
    let Ok(left_entity) = left_panel.single() else {
        return;
    };
    let Ok(right_entity) = right_panel.single() else {
        return;
    };

    commands.entity(left_entity).despawn_related::<Children>();
    commands.entity(right_entity).despawn_related::<Children>();

    super::multiplayer_tab::panels::build_multiplayer_panels(
        &mut commands,
        left_entity,
        right_entity,
        &lobby,
        &connection,
    );
}
