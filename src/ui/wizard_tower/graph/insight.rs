use bevy::prelude::*;

use super::super::constants::{INSIGHT_CONSTELLATION_OFFSET, INSIGHT_NODE_RADIUS};
use super::types::{InsightEdgeDef, InsightNodeDef};
use crate::game::insight_bonuses::InsightBonusStat;

/// Builds the insight bonus constellation: a diamond of 4 stat nodes around a central anchor.
///
/// Returns (nodes, edges, anchor_position).
pub(crate) fn build_insight_constellation() -> (Vec<InsightNodeDef>, Vec<InsightEdgeDef>, Vec2) {
    let center = INSIGHT_CONSTELLATION_OFFSET;
    let r = INSIGHT_NODE_RADIUS;

    // Diamond layout: top, right, bottom, left
    let positions = [
        (
            InsightBonusStat::SpellDamage,
            Vec2::new(center.x, center.y - r),
        ),
        (
            InsightBonusStat::CastSpeed,
            Vec2::new(center.x + r, center.y),
        ),
        (
            InsightBonusStat::ManaCost,
            Vec2::new(center.x, center.y + r),
        ),
        (
            InsightBonusStat::SpellRange,
            Vec2::new(center.x - r, center.y),
        ),
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
