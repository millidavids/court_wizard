use bevy::prelude::*;

use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;

/// Definition of a node in the spell graph.
pub(crate) struct SpellNodeDef {
    /// The spell this node represents, or `None` for the central anchor.
    pub spell: Option<Spell>,
    /// Graph-space coordinates, (0,0) = center.
    pub position: Vec2,
}

/// Definition of an edge in the spell graph.
pub(crate) struct SpellEdgeDef {
    /// Source spell (`None` = central anchor node).
    pub from_spell: Option<Spell>,
    /// Destination spell.
    pub to_spell: Spell,
    /// Precomputed waypoints for rendering (clipped to node edges, routed around obstacles).
    pub waypoints: Vec<Vec2>,
}

/// Definition of a node in the insight bonus constellation.
pub(crate) struct InsightNodeDef {
    pub stat: InsightBonusStat,
    pub position: Vec2,
}

/// Definition of an edge in the insight constellation.
pub(crate) struct InsightEdgeDef {
    pub start: Vec2,
    pub end: Vec2,
}
