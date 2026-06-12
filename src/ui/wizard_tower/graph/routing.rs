use bevy::prelude::*;

use super::super::constants::{
    GRAPH_EDGE_AVOIDANCE_MARGIN, GRAPH_EDGE_CURVE_SEGMENTS, GRAPH_FREE_NODE_SIZE, GRAPH_NODE_SIZE,
};
use super::types::{SpellEdgeDef, SpellNodeDef};
use crate::game::units::wizard::components::Spell;

/// Returns the visual radius for a node (half its rendered size).
pub(super) fn node_radius(spell: Option<Spell>) -> f32 {
    if spell.is_some() {
        GRAPH_NODE_SIZE / 2.0
    } else {
        GRAPH_FREE_NODE_SIZE / 2.0
    }
}

/// Finds the closest point on segment AB to point P.
fn closest_point_on_segment(a: Vec2, b: Vec2, p: Vec2) -> Vec2 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 0.001 {
        return a;
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    a + ab * t
}

/// Computes waypoints for each edge: clips endpoints to node borders and routes
/// around any intermediate nodes the line would visually intersect.
pub(super) fn compute_edge_waypoints(nodes: &[SpellNodeDef], edges: &mut [SpellEdgeDef]) {
    for edge in edges.iter_mut() {
        // Find positions of the two endpoints
        let from_pos = nodes
            .iter()
            .find(|n| n.spell == edge.from_spell)
            .map(|n| n.position)
            .unwrap_or(Vec2::ZERO);
        let to_pos = nodes
            .iter()
            .find(|n| n.spell == Some(edge.to_spell))
            .map(|n| n.position)
            .unwrap_or(Vec2::ZERO);

        let delta = to_pos - from_pos;
        let total_len = delta.length();
        if total_len < 0.1 {
            edge.waypoints = vec![from_pos, to_pos];
            continue;
        }
        let dir = delta / total_len;

        // Edges run center-to-center (nodes are opaque and rendered on top)
        let start = from_pos;
        let end = to_pos;

        // Check if any intermediate nodes are too close to the line
        let mut detours: Vec<(f32, Vec2)> = Vec::new();

        for node in nodes {
            // Skip the edge's own endpoints
            if node.spell == edge.from_spell || node.spell == Some(edge.to_spell) {
                continue;
            }

            let r = node_radius(node.spell) + GRAPH_EDGE_AVOIDANCE_MARGIN;
            let closest = closest_point_on_segment(start, end, node.position);
            let dist = (node.position - closest).length();

            if dist < r {
                // Need to route around this node — perpendicular offset
                let perp = Vec2::new(-dir.y, dir.x);
                // Choose the side that moves away from the node center
                let to_node = node.position - closest;
                let side = if to_node.dot(perp) > 0.0 { -1.0 } else { 1.0 };
                let waypoint = node.position + perp * side * (r + 4.0);

                // Track parameter along edge for ordering
                let t = (closest - start).dot(dir);
                detours.push((t, waypoint));
            }
        }

        // Sort detours by parameter along the edge direction
        detours.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Build raw waypoints then smooth into a curve if there are detours
        let mut raw = vec![start];
        for (_, wp) in &detours {
            raw.push(*wp);
        }
        raw.push(end);

        edge.waypoints = if raw.len() > 2 {
            smooth_waypoints(&raw)
        } else {
            raw
        };
    }
}

/// Subdivides a polyline with 3+ control points into a smooth Catmull-Rom curve.
/// Straight edges (2 points) should not be passed here.
fn smooth_waypoints(control_points: &[Vec2]) -> Vec<Vec2> {
    let n = control_points.len();
    debug_assert!(n >= 3);

    // Build extended control points for Catmull-Rom (mirror endpoints)
    let mut pts = Vec::with_capacity(n + 2);
    pts.push(control_points[0] * 2.0 - control_points[1]); // virtual start
    pts.extend_from_slice(control_points);
    pts.push(control_points[n - 1] * 2.0 - control_points[n - 2]); // virtual end

    let segments = n - 1; // number of spans between original control points
    let steps_per_span = GRAPH_EDGE_CURVE_SEGMENTS / segments;
    let steps = steps_per_span.max(4);

    let mut result = Vec::with_capacity(segments * steps + 1);

    // Evaluate Catmull-Rom for each span between consecutive original points
    for i in 0..segments {
        let p0 = pts[i];
        let p1 = pts[i + 1];
        let p2 = pts[i + 2];
        let p3 = pts[i + 3];

        let end = if i == segments - 1 { steps } else { steps - 1 };
        for s in 0..=end {
            let t = s as f32 / steps as f32;
            result.push(catmull_rom(p0, p1, p2, p3, t));
        }
    }

    result
}

/// Evaluates a Catmull-Rom spline at parameter t ∈ [0, 1] between p1 and p2.
fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}
