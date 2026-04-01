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
pub(crate) enum WizardTowerButtonAction {
    StudySpells,
    Compendium,
    StartNextBattle,
    ReturnToMenu,
    StartTimeTravel,
    #[cfg(debug_assertions)]
    DebugLevelUp,
    #[cfg(debug_assertions)]
    DebugLevelDown,
}

// ---------------------------------------------------------------------------
// Time Travel components
// ---------------------------------------------------------------------------

/// Marker for the outer collapsible time travel container (toggled visible/hidden).
#[derive(Component)]
pub(crate) struct TimeTravelContainer;

/// Marker for the scrollable time travel level list (used by handle_scroll).
#[derive(Component)]
pub(super) struct TimeTravelSection;

/// Clickable level entry in the time travel list.
#[derive(Component)]
pub(super) struct TimeTravelLevelButton(pub u32);

/// Text showing the currently selected time travel level.
#[derive(Component)]
pub(super) struct TimeTravelSelectedDisplay;

/// Tracks which level is selected in the time travel list.
#[derive(Resource, Default)]
pub(super) struct SelectedTimeTravelLevel(pub Option<u32>);

/// Marker for the level display text on the hub screen.
#[derive(Component)]
pub(crate) struct LevelDisplay;

/// Insight balance text on the hub.
#[derive(Component)]
pub(crate) struct InsightDisplay;

// ---------------------------------------------------------------------------
// Study screen
// ---------------------------------------------------------------------------

/// Marker for entities on the Study screen (despawned on exit Study).
#[derive(Component)]
pub(super) struct OnStudyScreen;

/// Actions from study screen buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StudyButtonAction {
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
    pub start_cursor: Vec2,
}

/// Resource driving an animated pan+zoom towards a target view state.
/// Inserted when a node is selected; removed when the animation completes or is interrupted.
#[derive(Resource)]
pub(super) struct GraphViewAnimation {
    pub target_offset: Vec2,
    pub target_scale: f32,
    /// Lerp speed (fraction of remaining distance per second).
    pub speed: f32,
}

/// Bounding box of the spell graph in graph-space coordinates.
/// Used to clamp pan/zoom so the user can't scroll beyond the node extents.
#[derive(Resource)]
pub(super) struct GraphBounds {
    pub min: Vec2,
    pub max: Vec2,
}

/// Marks the graph container for hit-testing and as the pan/zoom area.
#[derive(Component)]
pub(crate) struct SpellGraphArea;

/// Marks a spell node in the graph with its graph-space position.
#[derive(Component)]
pub(super) struct SpellGraphNode {
    pub spell: Spell,
    pub graph_position: Vec2,
}

/// Marks an edge segment entity with its two graph-space endpoints.
#[derive(Component)]
pub(super) struct SpellGraphEdge {
    pub start: Vec2,
    pub end: Vec2,
}

/// Marks the central anchor node in the graph.
#[derive(Component)]
pub(super) struct FreeNode;

/// Resource tracking the currently selected spell for the detail panel.
#[derive(Resource, Default)]
pub(super) struct SelectedStudySpell(pub Option<Spell>);

/// Marks the floating detail panel entity.
#[derive(Component)]
pub(crate) struct StudyDetailPanel;

/// Marks a detail panel's allocation slider track.
#[derive(Component)]
pub(super) struct StudyAllocationSlider {
    pub spell: Spell,
}

/// Marks the committed progress fill (non-draggable floor) in the unified slider.
#[derive(Component)]
pub(super) struct StudyProgressFill;

/// Marks the pending allocation fill (draggable region) in the unified slider.
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
// Talent UI components
// ---------------------------------------------------------------------------

/// Marker for the talent section container.
#[derive(Component)]
#[allow(dead_code)]
pub(super) struct TalentSection;

/// Marker for a clickable talent card.
#[derive(Component)]
pub(super) struct TalentCard {
    pub spell: Spell,
    pub tier: u8,
    pub choice: u8,
}

/// Marker for the talent progress bar fill.
#[derive(Component)]
pub(super) struct TalentProgressBarFill {
    #[allow(dead_code)]
    pub spell: Spell,
}

/// Marker for the talent description text area.
#[derive(Component)]
pub(super) struct TalentDescriptionText;

/// Resource tracking which talent card is hovered for description display.
#[derive(Resource, Default)]
#[allow(dead_code)]
pub(super) struct HoveredTalent(pub Option<(Spell, u8, u8)>);

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
