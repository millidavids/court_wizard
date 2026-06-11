use super::super::interaction::default_graph_offset;
use bevy::prelude::*;

use crate::game::resources::BattleInsightData;

use super::super::super::components::*;
use super::super::super::constants::*;
use super::super::super::materials::{
    ConcentricRingsMaterial, RadialProgressMaterial, StarSkyMaterial,
};
use super::graph_nav::animate_to_default_view;
use super::spawn_content::spawn_study_panels;

/// Marker for the debug "+10000 Insight" wrapper node so the global F2 debug
/// toggle can hide/show it.
#[cfg(debug_assertions)]
#[derive(Component)]
pub(crate) struct DebugInsightButton;

/// Inserted by `rebuild_study_ui` via deferred `Commands`. The deferred insert
/// makes the resource visible on the next frame — after the despawn/spawn
/// commands have flushed — so the layout systems re-run with the new entities
/// present. Consumed by `process_pending_graph_layout_refresh`.
#[derive(Resource, Default)]
pub(crate) struct PendingGraphLayoutRefresh;

/// Builds the study tab content into the wizard tower's left and right panels.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_study_panels(
    commands: &mut Commands,
    right_panel_entity: Entity,
    left_panel_entity: Entity,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    ring_materials: &mut Assets<ConcentricRingsMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
) {
    commands.insert_resource(InsightAllocation::default());
    // Zoom out and offset to show both spell web and insight constellation.
    commands.insert_resource(GraphViewState {
        offset: default_graph_offset(),
        scale: GRAPH_DEFAULT_SCALE,
    });
    commands.insert_resource(GraphDragState::default());
    commands.insert_resource(SelectedStudySpell::default());
    commands.insert_resource(SelectedInsightBonus::default());
    // Persist a layout-refresh marker until the freshly-spawned
    // `SpellGraphArea` has a non-zero `ComputedNode` size, so the position
    // systems pick up the real container dimensions on the first valid frame.
    commands.insert_resource(PendingGraphLayoutRefresh);

    spawn_study_panels(
        commands,
        right_panel_entity,
        left_panel_entity,
        battle_insight,
        asset_server,
        progress_materials,
        ring_materials,
        star_sky_materials,
    );
}

/// Cleans up study screen-specific resources when exiting the state.
pub(crate) fn cleanup_study_resources(mut commands: Commands) {
    commands.remove_resource::<InsightAllocation>();
    commands.remove_resource::<GraphViewState>();
    commands.remove_resource::<GraphDragState>();
    commands.remove_resource::<SelectedStudySpell>();
    commands.remove_resource::<SelectedInsightBonus>();
    commands.remove_resource::<GraphViewAnimation>();
    commands.remove_resource::<GraphBounds>();
    commands.remove_resource::<PendingGraphLayoutRefresh>();
}

/// Tears down and rebuilds the study screen UI.
///
/// `animate_to_default`: when true, animates the camera back to its default
/// pan/zoom. When false, the current `GraphViewState` is preserved.
///
/// `preserve_selection`: when true, keeps `SelectedStudySpell` intact across
/// the rebuild. The caller is expected to mark `SelectedStudySpell` as changed
/// afterward so `update_study_detail_panel` repopulates the left panel with
/// the spell's new (post-rebuild) state. When false, selection is cleared.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rebuild_study_ui(
    commands: &mut Commands,
    left_panel: &Query<Entity, With<super::super::super::layout::WizardTowerLeftPanel>>,
    right_panel: &Query<Entity, With<super::super::super::layout::WizardTowerRightPanel>>,
    selected: &mut Option<ResMut<SelectedStudySpell>>,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
    progress_materials: &mut Assets<RadialProgressMaterial>,
    ring_materials: &mut Assets<ConcentricRingsMaterial>,
    star_sky_materials: &mut Assets<StarSkyMaterial>,
    animate_to_default: bool,
    preserve_selection: bool,
) {
    let Ok(left_entity) = left_panel.single() else {
        return;
    };
    let Ok(right_entity) = right_panel.single() else {
        return;
    };
    commands.entity(left_entity).despawn_related::<Children>();
    commands.entity(right_entity).despawn_related::<Children>();
    commands.remove_resource::<InsightAllocation>();
    commands.insert_resource(InsightAllocation::default());
    commands.insert_resource(SelectedInsightBonus::default());
    // Deferred — visible next frame, after the despawn/spawn commands flush.
    // `process_pending_graph_layout_refresh` consumes it and triggers the
    // position systems to re-run on the freshly spawned entities.
    commands.insert_resource(PendingGraphLayoutRefresh);
    if !preserve_selection && let Some(sel) = selected {
        sel.0 = None;
    }
    if animate_to_default {
        animate_to_default_view(commands);
    } else {
        // Cancel any in-flight animation so it cannot drag the view to a
        // now-meaningless target after the rebuild.
        commands.remove_resource::<GraphViewAnimation>();
    }
    spawn_study_panels(
        commands,
        right_entity,
        left_entity,
        battle_insight,
        asset_server,
        progress_materials,
        ring_materials,
        star_sky_materials,
    );
}
