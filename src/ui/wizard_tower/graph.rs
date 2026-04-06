use bevy::prelude::*;

use super::constants::{
    GRAPH_EDGE_AVOIDANCE_MARGIN, GRAPH_EDGE_CURVE_SEGMENTS, GRAPH_FREE_NODE_SIZE, GRAPH_NODE_SIZE,
    INSIGHT_CONSTELLATION_OFFSET, INSIGHT_NODE_RADIUS,
};
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;

/// Definition of a node in the spell graph.
pub(super) struct SpellNodeDef {
    /// The spell this node represents, or `None` for the central anchor.
    pub spell: Option<Spell>,
    /// Graph-space coordinates, (0,0) = center.
    pub position: Vec2,
}

/// Definition of an edge in the spell graph.
pub(super) struct SpellEdgeDef {
    /// Source spell (`None` = central anchor node).
    pub from_spell: Option<Spell>,
    /// Destination spell.
    pub to_spell: Spell,
    /// Precomputed waypoints for rendering (clipped to node edges, routed around obstacles).
    pub waypoints: Vec<Vec2>,
}

/// Pushes overlapping nodes apart using iterative repulsion.
/// Preserves overall topology while ensuring minimum spacing.
fn separate_overlapping_nodes(nodes: &mut [SpellNodeDef]) {
    let min_distance: f32 = 90.0;

    for _ in 0..80 {
        let mut forces = vec![Vec2::ZERO; nodes.len()];
        let mut any_overlap = false;

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let delta = nodes[i].position - nodes[j].position;
                let dist = delta.length();
                if dist < min_distance {
                    any_overlap = true;
                    let overlap = min_distance - dist;
                    let dir = if dist > 0.1 {
                        delta / dist
                    } else {
                        // Nudge in a consistent direction for exactly overlapping nodes
                        Vec2::new(1.0, 0.3)
                    };
                    forces[i] += dir * overlap * 0.5;
                    forces[j] -= dir * overlap * 0.5;
                }
            }
        }

        if !any_overlap {
            break;
        }

        for (i, node) in nodes.iter_mut().enumerate() {
            // Don't move the center anchor node
            if node.spell.is_none() {
                continue;
            }
            node.position += forces[i];
        }
    }
}

/// Builds the full spell graph with node positions and edges.
///
/// Radial web layout centered at origin with 4 category roots:
///
/// - (0, 0): Central "Free" anchor node (visual hub, not a spell)
/// - Offense (MagicMissile): upper-right quadrant
/// - Control (Entangle): upper-left quadrant
/// - Support (GuardianCircle): lower-left quadrant
/// - Utility (Telekinesis): lower-right quadrant
///
/// Nodes are placed using polar coordinates (angle + distance from center)
/// so every branch radiates outward like a web.
pub(super) fn build_spell_graph() -> (Vec<SpellNodeDef>, Vec<SpellEdgeDef>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Place a node at (angle°, distance) from origin and connect it
    let add = |nodes: &mut Vec<SpellNodeDef>,
               edges: &mut Vec<SpellEdgeDef>,
               spell: Spell,
               angle_deg: f32,
               distance: f32,
               from: Option<Spell>| {
        let angle = angle_deg.to_radians();
        let pos = Vec2::new(angle.cos() * distance, angle.sin() * distance);
        nodes.push(SpellNodeDef {
            spell: Some(spell),
            position: pos,
        });
        edges.push(SpellEdgeDef {
            from_spell: from,
            to_spell: spell,
            waypoints: Vec::new(),
        });
    };

    // Ring distances from center
    let r1: f32 = 110.0; // Root category nodes
    let r2: f32 = 230.0; // Depth 2
    let r3: f32 = 350.0; // Depth 3
    let r4: f32 = 470.0; // Depth 4

    // Central anchor node
    nodes.push(SpellNodeDef {
        spell: None,
        position: Vec2::ZERO,
    });

    // -----------------------------------------------------------------------
    // Offense (upper-right quadrant, centered around -45°)
    // -----------------------------------------------------------------------
    add(&mut nodes, &mut edges, Spell::MagicMissile, -45.0, r1, None);

    // MagicMissile → 4 children fanning from -70° to -15°
    add(
        &mut nodes,
        &mut edges,
        Spell::PlagueWind,
        -68.0,
        r2,
        Some(Spell::MagicMissile),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Disintegrate,
        -53.0,
        r2,
        Some(Spell::MagicMissile),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::ChainLightning,
        -35.0,
        r2,
        Some(Spell::MagicMissile),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::FingerOfDeath,
        -15.0,
        r2,
        Some(Spell::MagicMissile),
    );

    // Disintegrate → Fireball → MeteorFall
    add(
        &mut nodes,
        &mut edges,
        Spell::Fireball,
        -56.0,
        r3,
        Some(Spell::Disintegrate),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::MeteorFall,
        -58.0,
        r4,
        Some(Spell::Fireball),
    );

    // ChainLightning → LightningRod
    add(
        &mut nodes,
        &mut edges,
        Spell::LightningRod,
        -32.0,
        r3,
        Some(Spell::ChainLightning),
    );

    // FingerOfDeath → MarkOfDeath
    add(
        &mut nodes,
        &mut edges,
        Spell::MarkOfDeath,
        -12.0,
        r3,
        Some(Spell::FingerOfDeath),
    );

    // -----------------------------------------------------------------------
    // Control (upper-left quadrant, centered around -135°)
    // -----------------------------------------------------------------------
    add(&mut nodes, &mut edges, Spell::Entangle, -135.0, r1, None);

    // Entangle → 3 children fanning from -158° to -115°
    add(
        &mut nodes,
        &mut edges,
        Spell::Grease,
        -157.0,
        r2,
        Some(Spell::Entangle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::SpikeGrowth,
        -135.0,
        r2,
        Some(Spell::Entangle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Sleep,
        -116.0,
        r2,
        Some(Spell::Entangle),
    );

    // Grease → WallOfFire
    add(
        &mut nodes,
        &mut edges,
        Spell::WallOfFire,
        -160.0,
        r3,
        Some(Spell::Grease),
    );

    // SpikeGrowth → WallOfStone, Squall
    add(
        &mut nodes,
        &mut edges,
        Spell::WallOfStone,
        -141.0,
        r3,
        Some(Spell::SpikeGrowth),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Squall,
        -129.0,
        r3,
        Some(Spell::SpikeGrowth),
    );

    // Sleep → MindControl, Polymorph → BlackHole
    add(
        &mut nodes,
        &mut edges,
        Spell::MindControl,
        -122.0,
        r3,
        Some(Spell::Sleep),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Polymorph,
        -110.0,
        r3,
        Some(Spell::Sleep),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::BlackHole,
        -108.0,
        r4,
        Some(Spell::Polymorph),
    );

    // -----------------------------------------------------------------------
    // Support (lower-left quadrant, centered around 135°)
    // -----------------------------------------------------------------------
    add(
        &mut nodes,
        &mut edges,
        Spell::GuardianCircle,
        135.0,
        r1,
        None,
    );

    // GuardianCircle → 3 children fanning from 115° to 158°
    add(
        &mut nodes,
        &mut edges,
        Spell::BattleHymn,
        118.0,
        r2,
        Some(Spell::GuardianCircle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::FogCloud,
        140.0,
        r2,
        Some(Spell::GuardianCircle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::BerserkerRage,
        158.0,
        r2,
        Some(Spell::GuardianCircle),
    );

    // BattleHymn → HealingPlume, Haste
    add(
        &mut nodes,
        &mut edges,
        Spell::HealingPlume,
        110.0,
        r3,
        Some(Spell::BattleHymn),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Haste,
        125.0,
        r3,
        Some(Spell::BattleHymn),
    );

    // FogCloud → Teleport
    add(
        &mut nodes,
        &mut edges,
        Spell::Teleport,
        143.0,
        r3,
        Some(Spell::FogCloud),
    );

    // BerserkerRage → RaiseTheDead
    add(
        &mut nodes,
        &mut edges,
        Spell::RaiseTheDead,
        162.0,
        r3,
        Some(Spell::BerserkerRage),
    );

    // -----------------------------------------------------------------------
    // Utility (lower-right quadrant, centered around 45°)
    // -----------------------------------------------------------------------
    add(&mut nodes, &mut edges, Spell::Telekinesis, 45.0, r1, None);

    // Telekinesis → 3 children fanning from 28° to 62°
    add(
        &mut nodes,
        &mut edges,
        Spell::Dispel,
        28.0,
        r2,
        Some(Spell::Telekinesis),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Banishment,
        45.0,
        r2,
        Some(Spell::Telekinesis),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::ArcaneCrystal,
        62.0,
        r2,
        Some(Spell::Telekinesis),
    );

    // Push apart any nodes that overlap
    separate_overlapping_nodes(&mut nodes);

    // Compute edge waypoints (clipped to node edges, routed around obstacles)
    compute_edge_waypoints(&nodes, &mut edges);

    (nodes, edges)
}

// ---------------------------------------------------------------------------
// Insight constellation layout
// ---------------------------------------------------------------------------

/// Definition of a node in the insight bonus constellation.
pub(super) struct InsightNodeDef {
    pub stat: InsightBonusStat,
    pub position: Vec2,
}

/// Definition of an edge in the insight constellation.
pub(super) struct InsightEdgeDef {
    pub start: Vec2,
    pub end: Vec2,
}

/// Builds the insight bonus constellation: a diamond of 4 stat nodes around a central anchor.
///
/// Returns (nodes, edges, anchor_position).
pub(super) fn build_insight_constellation() -> (Vec<InsightNodeDef>, Vec<InsightEdgeDef>, Vec2) {
    let center = INSIGHT_CONSTELLATION_OFFSET;
    let r = INSIGHT_NODE_RADIUS;

    // Diamond layout: top, right, bottom, left
    let positions = [
        (InsightBonusStat::SpellDamage, Vec2::new(center.x, center.y - r)),
        (InsightBonusStat::CastSpeed, Vec2::new(center.x + r, center.y)),
        (InsightBonusStat::ManaCost, Vec2::new(center.x, center.y + r)),
        (InsightBonusStat::SpellRange, Vec2::new(center.x - r, center.y)),
    ];

    let nodes: Vec<InsightNodeDef> = positions
        .iter()
        .map(|(stat, pos)| InsightNodeDef {
            stat: *stat,
            position: *pos,
        })
        .collect();

    let edges: Vec<InsightEdgeDef> = positions
        .iter()
        .map(|(_, pos)| InsightEdgeDef {
            start: center,
            end: *pos,
        })
        .collect();

    (nodes, edges, center)
}

// ---------------------------------------------------------------------------
// Spell graph helpers
// ---------------------------------------------------------------------------

/// Returns the visual radius for a node (half its rendered size).
fn node_radius(spell: Option<Spell>) -> f32 {
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
fn compute_edge_waypoints(nodes: &[SpellNodeDef], edges: &mut [SpellEdgeDef]) {
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
