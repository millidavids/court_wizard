use bevy::prelude::*;

use crate::config::save_data::load_unified_save;

use super::super::super::components::*;
use super::super::super::constants::*;
use super::helpers::{clip_line_to_rect, graph_to_screen, is_prereq_met_in, is_spell_unlocked_in};

/// Updates the screen position of all graph nodes based on pan/zoom state.
pub(crate) fn update_graph_node_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut node_query: Query<(&mut Node, &SpellGraphNode), Without<FreeNode>>,
    mut free_node_query: Query<&mut Node, (With<FreeNode>, Without<SpellGraphNode>)>,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() * computed.inverse_scale_factor() / 2.0;

    for (mut node, graph_node) in &mut node_query {
        let screen_pos = graph_to_screen(graph_node.graph_position, &view, container_center);
        let scaled_size = GRAPH_NODE_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }

    for mut node in &mut free_node_query {
        let screen_pos = graph_to_screen(Vec2::ZERO, &view, container_center);
        let scaled_size = GRAPH_FREE_NODE_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }
}

/// Updates border colors on spell nodes based on the selected spell.
/// Loads save data once per invocation (the system is change-detection gated).
pub(crate) fn update_graph_node_borders(
    selected: Res<SelectedStudySpell>,
    mut border_query: Query<(&mut BorderColor, &SpellGraphNode)>,
) {
    // Load save data once for the whole pass rather than once per node.
    let save = load_unified_save();
    let unlocked_names: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    for (mut border_color, graph_node) in &mut border_query {
        if selected.0 == Some(graph_node.spell) {
            *border_color = BorderColor::all(GRAPH_NODE_SELECTED_BORDER);
        } else {
            let spell = graph_node.spell;
            let unlocked = is_spell_unlocked_in(spell, &unlocked_names);
            let prereq_met = is_prereq_met_in(spell, &unlocked_names);
            let cost = spell.research_cost();
            let is_free = cost == 0;

            let border = if is_free || unlocked {
                GRAPH_NODE_COMPLETED_BORDER
            } else if prereq_met {
                GRAPH_NODE_BORDER
            } else {
                GRAPH_NODE_LOCKED_BORDER
            };
            *border_color = BorderColor::all(border);
        }
    }
}

/// Updates graph edge segment positions based on pan/zoom state.
/// Each segment is a rotated rectangle connecting two consecutive waypoints.
pub(crate) fn update_graph_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut segments: Query<(
        &mut Node,
        &mut UiTransform,
        &mut Visibility,
        &SpellGraphEdge,
    )>,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_size = computed.size() * computed.inverse_scale_factor();
    let container_center = container_size / 2.0;
    let thickness = (GRAPH_EDGE_THICKNESS * view.scale).max(1.0);

    for (mut node, mut ui_transform, mut vis, edge) in &mut segments {
        let screen_a = graph_to_screen(edge.start, &view, container_center);
        let screen_b = graph_to_screen(edge.end, &view, container_center);

        // Clip the line segment to the graph area bounds so the rotated
        // rectangle stays within the container. UiTransform rotation
        // bypasses Overflow::clip, so we must clip geometry manually.
        let clip = Vec2::ZERO..=container_size;
        if let Some((ca, cb)) = clip_line_to_rect(screen_a, screen_b, &clip) {
            *vis = Visibility::Inherited;
            let delta = cb - ca;
            let length = delta.length();
            let angle = delta.y.atan2(delta.x);
            let midpoint = (ca + cb) / 2.0;
            node.left = Val::Px(midpoint.x - length / 2.0);
            node.top = Val::Px(midpoint.y - thickness / 2.0);
            node.width = Val::Px(length);
            node.height = Val::Px(thickness);
            ui_transform.rotation = Rot2::radians(angle);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// Updates the screen position of insight bonus nodes and the constellation anchor.
#[allow(clippy::type_complexity)]
pub(crate) fn update_insight_node_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut inode_query: Query<
        (&mut Node, &InsightBonusNode),
        (
            Without<InsightConstellationAnchor>,
            Without<SpellGraphNode>,
            Without<FreeNode>,
        ),
    >,
    mut anchor_query: Query<
        &mut Node,
        (
            With<InsightConstellationAnchor>,
            Without<InsightBonusNode>,
            Without<SpellGraphNode>,
            Without<FreeNode>,
        ),
    >,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_center = computed.size() * computed.inverse_scale_factor() / 2.0;

    for (mut node, inode) in &mut inode_query {
        let screen_pos = graph_to_screen(inode.graph_position, &view, container_center);
        let scaled_size = INSIGHT_NODE_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }

    for mut node in &mut anchor_query {
        let screen_pos = graph_to_screen(INSIGHT_CONSTELLATION_OFFSET, &view, container_center);
        let scaled_size = INSIGHT_ANCHOR_SIZE * view.scale;
        node.left = Val::Px(screen_pos.x - scaled_size / 2.0);
        node.top = Val::Px(screen_pos.y - scaled_size / 2.0);
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
    }
}

/// Updates insight constellation edge positions based on pan/zoom state.
pub(crate) fn update_insight_edge_positions(
    view: Res<GraphViewState>,
    graph_area_query: Query<&ComputedNode, With<SpellGraphArea>>,
    mut segments: Query<
        (
            &mut Node,
            &mut UiTransform,
            &mut Visibility,
            &InsightConstellationEdge,
        ),
        Without<SpellGraphEdge>,
    >,
) {
    let Ok(computed) = graph_area_query.single() else {
        return;
    };
    let container_size = computed.size() * computed.inverse_scale_factor();
    let container_center = container_size / 2.0;
    let thickness = (GRAPH_EDGE_THICKNESS * view.scale).max(1.0);

    for (mut node, mut ui_transform, mut vis, edge) in &mut segments {
        let screen_a = graph_to_screen(edge.start, &view, container_center);
        let screen_b = graph_to_screen(edge.end, &view, container_center);

        let clip = Vec2::ZERO..=container_size;
        if let Some((ca, cb)) = clip_line_to_rect(screen_a, screen_b, &clip) {
            *vis = Visibility::Inherited;
            let delta = cb - ca;
            let length = delta.length();
            let angle = delta.y.atan2(delta.x);
            let midpoint = (ca + cb) / 2.0;
            node.left = Val::Px(midpoint.x - length / 2.0);
            node.top = Val::Px(midpoint.y - thickness / 2.0);
            node.width = Val::Px(length);
            node.height = Val::Px(thickness);
            ui_transform.rotation = Rot2::radians(angle);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// Updates border colors on insight bonus nodes based on selection state.
pub(crate) fn update_insight_node_borders(
    selected: Res<SelectedInsightBonus>,
    mut border_query: Query<(&mut BorderColor, &InsightBonusNode)>,
) {
    let bonuses = crate::config::save_data::get_all_insight_bonuses();
    let max = crate::game::insight_bonuses::InsightBonusStat::max_level();
    for (mut border_color, inode) in &mut border_query {
        if selected.0 == Some(inode.stat) {
            *border_color = BorderColor::all(GRAPH_NODE_SELECTED_BORDER);
        } else {
            let level = bonuses.get(inode.stat.id()).copied().unwrap_or(0).min(max);
            let border = if level >= max {
                INSIGHT_NODE_MAXED_BORDER
            } else {
                INSIGHT_NODE_BORDER
            };
            *border_color = BorderColor::all(border);
        }
    }
}
