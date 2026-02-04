//! Pathfinding components.

use bevy::prelude::*;

/// Component that determines which flow field a unit should follow.
#[derive(Component, Clone, Copy)]
pub struct FlowFieldInfluence {
    /// Type of flow field to follow.
    pub field_type: FlowFieldType,
}

/// Types of flow fields available.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowFieldType {
    /// Attackers flow toward the King.
    Attacker,
    /// King defenders flow toward their rally point (closest to action).
    KingDefender,
    /// Infantry defenders flow toward their rally point (middle line).
    InfantryDefender,
    /// Archer defenders flow toward their rally point (back line).
    ArcherDefender,
}
