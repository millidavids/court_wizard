use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Shared (both Main and Study screens)
// ---------------------------------------------------------------------------

/// Marker for entities that should be despawned when exiting WizardTower state entirely.
#[derive(Component)]
pub(super) struct OnWizardTowerScreen;

// ---------------------------------------------------------------------------
// Main hub screen
// ---------------------------------------------------------------------------

/// Marker for entities on the Main hub screen (despawned on exit Main).
#[derive(Component)]
pub(super) struct OnMainScreen;

/// Actions from hub buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum WizardTowerButtonAction {
    StudySpells,
    StartNextBattle,
    ReturnToMenu,
    #[cfg(debug_assertions)]
    DebugLevelUp,
    #[cfg(debug_assertions)]
    DebugLevelDown,
}

/// Marker for the level display text on the hub screen (for reactive updates).
#[cfg(debug_assertions)]
#[derive(Component)]
pub(super) struct LevelDisplay;

/// Insight balance text on the hub.
#[derive(Component)]
pub(super) struct InsightDisplay;

// ---------------------------------------------------------------------------
// Study screen
// ---------------------------------------------------------------------------

/// Marker for entities on the Study screen (despawned on exit Study).
#[derive(Component)]
pub(super) struct OnStudyScreen;

/// Actions from study screen buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum StudyButtonAction {
    Commit,
    Back,
    #[cfg(debug_assertions)]
    DebugGrantInsight,
}

/// Pending Insight allocation display in study header.
#[derive(Component)]
pub(super) struct PendingInsightDisplay;

/// Insight balance display in study header.
#[derive(Component)]
pub(super) struct StudyInsightDisplay;

// ---------------------------------------------------------------------------
// Graph components and resources
// ---------------------------------------------------------------------------

/// Resource tracking the current pan offset and zoom scale of the spell graph.
#[derive(Resource)]
pub(super) struct GraphViewState {
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
pub(super) struct GraphDragState {
    pub dragging: bool,
    pub last_cursor: Vec2,
}

/// Marks the graph container for hit-testing and as the pan/zoom area.
#[derive(Component)]
pub(super) struct SpellGraphArea;

/// Marks a spell node in the graph with its graph-space position.
#[derive(Component)]
pub(super) struct SpellGraphNode {
    pub spell: Spell,
    pub graph_position: Vec2,
}

/// Marks an edge entity connecting two nodes in the graph.
#[derive(Component)]
pub(super) struct SpellGraphEdge {
    pub from_spell: Option<Spell>,
    pub to_spell: Spell,
}

/// Marks the horizontal segment of an L-shaped edge connector.
#[derive(Component)]
pub(super) struct EdgeSegmentH;

/// Marks the vertical segment of an L-shaped edge connector.
#[derive(Component)]
pub(super) struct EdgeSegmentV;

/// Marks the central anchor node in the graph.
#[derive(Component)]
pub(super) struct FreeNode;

/// Resource tracking the currently selected spell for the detail panel.
#[derive(Resource, Default)]
pub(super) struct SelectedStudySpell(pub Option<Spell>);

/// Marks the floating detail panel entity.
#[derive(Component)]
pub(super) struct StudyDetailPanel;

/// Marks a detail panel's allocation slider track.
#[derive(Component)]
pub(super) struct StudyAllocationSlider {
    pub spell: Spell,
}

/// Marks a detail panel's allocation slider fill.
#[derive(Component)]
pub(super) struct StudyAllocationFill {
    pub spell: Spell,
}

/// Marks a detail panel's allocation slider handle.
#[derive(Component)]
pub(super) struct StudyAllocationHandle {
    pub spell: Spell,
    pub is_dragging: bool,
}

/// Text showing allocation progress in the detail panel.
#[derive(Component)]
pub(super) struct StudyAllocationText {
    pub spell: Spell,
}

// ---------------------------------------------------------------------------
// Allocation resource
// ---------------------------------------------------------------------------

/// Resource tracking pending Insight allocations before committing.
/// Only exists while in MetaGameState::Study.
#[derive(Resource, Default)]
pub(super) struct InsightAllocation {
    /// Spell → how much Insight the player wants to invest (before affinity bonus).
    pub allocations: HashMap<Spell, u32>,
}

impl InsightAllocation {
    /// Total Insight allocated across all spells.
    pub fn total_allocated(&self) -> u32 {
        self.allocations.values().sum()
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
}
