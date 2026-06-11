use bevy::input::mouse::{MouseButton, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseClicked;

use super::super::super::components::*;
use super::super::super::constants::*;
use super::super::interaction::default_graph_offset;
use super::spawn::PendingGraphLayoutRefresh;

/// Detects clicks on spell graph nodes and insight bonus nodes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_graph_node_clicks(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    node_query: Query<&SpellGraphNode>,
    insight_node_query: Query<&InsightBonusNode>,
    mut selected: ResMut<SelectedStudySpell>,
    mut selected_insight: ResMut<SelectedInsightBonus>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    for event in button_clicked.read() {
        // Check spell nodes first
        if let Ok(node) = node_query.get(event.button) {
            selected_insight.0 = None; // Clear insight selection
            if selected.0 == Some(node.spell) {
                selected.0 = None;
                animate_to_default_view(&mut commands);
            } else {
                selected.0 = Some(node.spell);
                animate_to_node(&mut commands, &graph_area_query, node.graph_position);
            }
            continue;
        }

        // Check insight bonus nodes
        if let Ok(inode) = insight_node_query.get(event.button) {
            selected.0 = None; // Clear spell selection
            if selected_insight.0 == Some(inode.stat) {
                selected_insight.0 = None;
                animate_to_default_view(&mut commands);
            } else {
                selected_insight.0 = Some(inode.stat);
                animate_to_node(&mut commands, &graph_area_query, inode.graph_position);
            }
        }
    }
}

/// Inserts the `GraphViewAnimation` that returns the graph to the "both
/// clusters visible" default — used by deselect paths (B button, re-clicking
/// the selected node, Commit rebuild).
pub(crate) fn animate_to_default_view(commands: &mut Commands) {
    commands.insert_resource(GraphViewAnimation {
        target_offset: default_graph_offset(),
        target_scale: GRAPH_DEFAULT_SCALE,
        speed: GRAPH_ANIMATION_SPEED,
    });
}

/// Animates the graph view to center a node in the right 2/3 of the screen.
pub(crate) fn animate_to_node(
    commands: &mut Commands,
    graph_area_query: &Query<&ComputedNode, With<SpellGraphArea>>,
    graph_position: Vec2,
) {
    if let Ok(computed) = graph_area_query.single() {
        let size = computed.size() * computed.inverse_scale_factor();
        let container_center = size / 2.0;
        // Center the node in the graph area (detail panel is in a separate left panel now)
        let target = container_center;
        let target_scale = GRAPH_ZOOM_MAX;
        let target_offset = target - container_center - graph_position * target_scale;
        commands.insert_resource(GraphViewAnimation {
            target_offset,
            target_scale,
            speed: GRAPH_ANIMATION_SPEED,
        });
    }
}

/// Clamps the view offset so the outermost graph nodes can reach roughly
/// the screen center but no further.
pub(crate) fn clamp_view_offset(view: &mut GraphViewState, bounds: &GraphBounds) {
    let margin = GRAPH_NODE_SIZE;
    // When panning, stop when the outermost node reaches near the center.
    // offset.x range: [-max_x * scale - margin, -min_x * scale + margin]
    let min_x = -bounds.max.x * view.scale - margin;
    let max_x = -bounds.min.x * view.scale + margin;
    let min_y = -bounds.max.y * view.scale - margin;
    let max_y = -bounds.min.y * view.scale + margin;

    view.offset.x = view.offset.x.clamp(min_x, max_x);
    view.offset.y = view.offset.y.clamp(min_y, max_y);
}

/// Handles panning the graph via left-click drag on the background.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_graph_pan(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    ui_scale: Res<bevy::ui::UiScale>,
    mut view: ResMut<GraphViewState>,
    mut drag: ResMut<GraphDragState>,
    bounds: Option<Res<GraphBounds>>,
    mut selected: ResMut<SelectedStudySpell>,
    mut selected_insight: ResMut<SelectedInsightBonus>,
    node_interactions: Query<&Interaction, Or<(With<SpellGraphNode>, With<InsightBonusNode>)>>,
    graph_area_interaction: Query<&Interaction, With<SpellGraphArea>>,
    slider_interactions: Query<
        &Interaction,
        Or<(
            With<StudyAllocationSlider>,
            With<StudyAllocationHandle>,
            With<InsightBonusSlider>,
            With<InsightBonusSliderHandle>,
        )>,
    >,
) {
    let Some(cursor_pos) = corrected_cursor.0 else {
        if drag.dragging {
            drag.dragging = false;
        }
        return;
    };
    // Convert window-logical cursor to UI space
    let cursor_ui = cursor_pos / ui_scale.0;

    // Only interact with pan/deselect when cursor is over the graph area.
    // Clicks on the left panel (detail panel, talents, etc.) must not affect
    // the right panel's graph state.
    let cursor_over_graph = graph_area_interaction
        .iter()
        .any(|interaction| *interaction != Interaction::None);

    if buttons.just_pressed(MouseButton::Left) {
        // Don't start dragging if a node or slider is being pressed,
        // or if the cursor isn't over the graph area
        let any_node_pressed = node_interactions.iter().any(|i| *i == Interaction::Pressed);
        let slider_pressed = slider_interactions.iter().any(|i| *i != Interaction::None);
        if !any_node_pressed && cursor_over_graph && !slider_pressed {
            drag.dragging = true;
            drag.last_cursor = cursor_ui;
            drag.start_cursor = cursor_ui;
            commands.remove_resource::<GraphViewAnimation>();
        }
    }

    if buttons.just_released(MouseButton::Left) && drag.dragging {
        let total_moved = (cursor_ui - drag.start_cursor).length();
        // Deselect on a click (not a drag) on empty space within the graph area
        if total_moved < 4.0 {
            selected.0 = None;
            selected_insight.0 = None;
        }
        drag.dragging = false;
        return;
    }

    if !buttons.pressed(MouseButton::Left) {
        drag.dragging = false;
        return;
    }

    if drag.dragging {
        let delta = cursor_ui - drag.last_cursor;
        view.offset += delta;
        if let Some(bounds) = &bounds {
            clamp_view_offset(&mut view, bounds);
        }
        drag.last_cursor = cursor_ui;
    }
}

/// Handles zooming the graph via mouse scroll wheel.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_graph_zoom(
    mut commands: Commands,
    mut mouse_wheel: MessageReader<MouseWheel>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    ui_scale: Res<bevy::ui::UiScale>,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area_query: Query<
        (&ComputedNode, &bevy::ui::ui_transform::UiGlobalTransform),
        With<SpellGraphArea>,
    >,
) {
    let Some(cursor_pos) = corrected_cursor.0 else {
        return;
    };
    let Ok((computed, ui_transform)) = graph_area_query.single() else {
        return;
    };
    let cursor_ui = cursor_pos / ui_scale.0;
    let isf = computed.inverse_scale_factor();
    // Absolute center of the graph area in UI space
    let container_abs_center =
        Vec2::new(ui_transform.translation.x, ui_transform.translation.y) * isf;

    for event in mouse_wheel.read() {
        let scroll_delta = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        };

        let old_scale = view.scale;
        let new_scale = (old_scale * (1.0 + scroll_delta * GRAPH_ZOOM_SPEED))
            .clamp(GRAPH_ZOOM_MIN, GRAPH_ZOOM_MAX);

        if (new_scale - old_scale).abs() > f32::EPSILON {
            // Cancel any running animation
            commands.remove_resource::<GraphViewAnimation>();
            // Adjust offset to keep point under cursor stationary
            let cursor_from_center = cursor_ui - container_abs_center;
            let graph_point = (cursor_from_center - view.offset) / old_scale;
            view.offset = cursor_from_center - graph_point * new_scale;
            view.scale = new_scale;
            if let Some(bounds) = &bounds {
                clamp_view_offset(&mut view, bounds);
            }
        }
    }
}

/// Smoothly animates the graph view toward a target offset and zoom.
/// Removed automatically when the animation reaches its destination.
pub(crate) fn animate_graph_view(
    mut commands: Commands,
    time: Res<Time>,
    animation: Option<Res<GraphViewAnimation>>,
    bounds: Option<Res<GraphBounds>>,
    mut view: ResMut<GraphViewState>,
) {
    let Some(anim) = animation else {
        return;
    };

    let t = (anim.speed * time.delta_secs()).min(1.0);
    view.offset = view.offset.lerp(anim.target_offset, t);
    view.scale = view.scale + (anim.target_scale - view.scale) * t;

    if let Some(bounds) = &bounds {
        clamp_view_offset(&mut view, bounds);
    }

    // Stop when close enough
    let offset_dist = (view.offset - anim.target_offset).length();
    let scale_dist = (view.scale - anim.target_scale).abs();
    if offset_dist < 0.5 && scale_dist < 0.001 {
        view.offset = anim.target_offset;
        view.scale = anim.target_scale;
        commands.remove_resource::<GraphViewAnimation>();
    }
}

/// Marks `GraphViewState` as changed after a panel rebuild so the position
/// systems re-run on freshly spawned nodes. The direct `set_changed()` call
/// in `handle_study_button_actions` would fire in the same tick as the
/// deferred spawn commands — before the new entities exist.
///
/// The marker persists across frames until the `SpellGraphArea`'s
/// `ComputedNode` reports a non-zero size — Bevy's UI layout pass may take
/// one or more frames to lay out a freshly spawned area, and `container_center`
/// derived from a zero size would place every node off-screen to the left.
pub(crate) fn process_pending_graph_layout_refresh(
    mut commands: Commands,
    mut view: ResMut<GraphViewState>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    view.set_changed();
    let area_ready = graph_area
        .single()
        .map(|computed| computed.size().length_squared() > 0.0)
        .unwrap_or(false);
    if area_ready {
        commands.remove_resource::<PendingGraphLayoutRefresh>();
    }
}
