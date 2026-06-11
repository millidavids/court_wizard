use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Graph components and resources
// ---------------------------------------------------------------------------

/// Resource tracking the current pan offset and zoom scale of the spell graph.
#[derive(Resource)]
pub(crate) struct GraphViewState {
    /// Pan offset in graph-space pixels.
    pub offset: Vec2,
    /// Zoom scale (1.0 = default).
    pub scale: f32,
}

impl Default for GraphViewState {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            scale: 1.0,
        }
    }
}

/// Resource tracking drag state for panning the graph.
#[derive(Resource, Default)]
pub(crate) struct GraphDragState {
    pub dragging: bool,
    pub last_cursor: Vec2,
    pub start_cursor: Vec2,
}

/// Resource driving an animated pan+zoom towards a target view state.
/// Inserted when a node is selected; removed when the animation completes or is interrupted.
#[derive(Resource)]
pub(crate) struct GraphViewAnimation {
    pub target_offset: Vec2,
    pub target_scale: f32,
    /// Lerp speed (fraction of remaining distance per second).
    pub speed: f32,
}

/// Bounding box of the spell graph in graph-space coordinates.
/// Used to clamp pan/zoom so the user can't scroll beyond the node extents.
#[derive(Resource)]
pub(crate) struct GraphBounds {
    pub min: Vec2,
    pub max: Vec2,
}

/// Marks the graph container for hit-testing and as the pan/zoom area.
#[derive(Component)]
pub(crate) struct SpellGraphArea;

/// Marks a spell node in the graph with its graph-space position.
#[derive(Component)]
pub(crate) struct SpellGraphNode {
    pub spell: Spell,
    pub graph_position: Vec2,
}

/// Marks an edge segment entity with its two graph-space endpoints.
#[derive(Component)]
pub(crate) struct SpellGraphEdge {
    pub start: Vec2,
    pub end: Vec2,
}

/// Marks the central anchor node in the graph.
#[derive(Component)]
pub(crate) struct FreeNode;

/// Resource tracking the currently selected spell for the detail panel.
#[derive(Resource, Default)]
pub(crate) struct SelectedStudySpell(pub Option<Spell>);

// ---------------------------------------------------------------------------
// Insight constellation components
// ---------------------------------------------------------------------------

/// Marks an insight bonus node in the constellation with its graph-space position.
#[derive(Component)]
pub(crate) struct InsightBonusNode {
    pub stat: InsightBonusStat,
    pub graph_position: Vec2,
}

/// Marks the central anchor of the insight constellation.
#[derive(Component)]
pub(crate) struct InsightConstellationAnchor;

/// Marks an edge segment in the insight constellation.
#[derive(Component)]
pub(crate) struct InsightConstellationEdge {
    pub start: Vec2,
    pub end: Vec2,
}

/// Resource tracking the currently selected insight bonus for the detail panel.
/// Mutually exclusive with `SelectedStudySpell`.
#[derive(Resource, Default)]
pub(crate) struct SelectedInsightBonus(pub Option<InsightBonusStat>);

/// Marks the slider track for an insight bonus allocation.
#[derive(Component)]
pub(crate) struct InsightBonusSlider {
    pub stat: InsightBonusStat,
}

/// Marks the committed progress fill in an insight bonus slider.
#[derive(Component)]
pub(crate) struct InsightBonusProgressFill;

/// Marks the pending allocation fill in an insight bonus slider.
#[derive(Component)]
pub(crate) struct InsightBonusAllocationFill {
    pub stat: InsightBonusStat,
}

/// Marks the slider handle for an insight bonus allocation.
#[derive(Component)]
pub(crate) struct InsightBonusSliderHandle {
    pub stat: InsightBonusStat,
    pub is_dragging: bool,
}

/// Text showing allocation progress for an insight bonus.
#[derive(Component)]
pub(crate) struct InsightBonusAllocationText {
    pub stat: InsightBonusStat,
}

/// Marks a concentric rings material node on an insight bonus graph node.
#[derive(Component)]
pub(crate) struct InsightBonusRings {
    pub stat: InsightBonusStat,
}

/// Marks a text label inside a graph node that should scale with zoom.
#[derive(Component)]
pub(crate) struct GraphNodeLabel {
    pub base_size: f32,
}

// ---------------------------------------------------------------------------
// Allocation resource
// ---------------------------------------------------------------------------

/// Resource tracking pending Insight allocations before committing.
/// Only exists while in MetaGameState::Study.
#[derive(Resource, Default)]
pub(crate) struct InsightAllocation {
    /// Spell → how much Insight the player wants to invest (before affinity bonus).
    pub allocations: HashMap<Spell, u32>,
    /// Insight bonus stat → how much Insight the player wants to invest.
    pub bonus_allocations: HashMap<InsightBonusStat, u32>,
}

impl InsightAllocation {
    /// Total Insight allocated across all spells and bonuses.
    pub fn total_allocated(&self) -> u32 {
        self.allocations.values().sum::<u32>() + self.bonus_allocations.values().sum::<u32>()
    }

    /// Get the allocation for a specific spell.
    pub fn get(&self, spell: &Spell) -> u32 {
        self.allocations.get(spell).copied().unwrap_or(0)
    }

    /// Set allocation for a specific spell. Removes entry if 0.
    pub fn set(&mut self, spell: Spell, amount: u32) {
        if amount == 0 {
            self.allocations.remove(&spell);
        } else {
            self.allocations.insert(spell, amount);
        }
    }

    /// Get the allocation for a specific insight bonus stat.
    pub fn get_bonus(&self, stat: &InsightBonusStat) -> u32 {
        self.bonus_allocations.get(stat).copied().unwrap_or(0)
    }

    /// Set allocation for a specific insight bonus stat. Removes entry if 0.
    pub fn set_bonus(&mut self, stat: InsightBonusStat, amount: u32) {
        if amount == 0 {
            self.bonus_allocations.remove(&stat);
        } else {
            self.bonus_allocations.insert(stat, amount);
        }
    }
}
