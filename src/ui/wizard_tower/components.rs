use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Shared (both Main and Study screens)
// ---------------------------------------------------------------------------

/// Marker for entities that should be despawned when exiting WizardTower state entirely.
#[derive(Component)]
pub(super) struct OnWizardTowerScreen;

/// Marker for the header and content rows (toggled by F3 debug).
#[cfg(debug_assertions)]
#[derive(Component)]
pub(super) struct WizardTowerUiContent;

/// Marker for the arcane rune background MaterialNode.
#[derive(Component)]
pub(super) struct ArcaneRuneBackground;

/// Orbiting spell name text around the arcane rune circles.
#[derive(Component)]
pub(super) struct ArcaneRuneText {
    /// Base angle in radians (position on the circle at t=0).
    pub angle: f32,
    /// Radius as a fraction of container height (matches shader UV space).
    pub radius: f32,
    /// Rotation speed in radians per second (positive = CW).
    pub speed: f32,
}

// ---------------------------------------------------------------------------
// Main hub screen
// ---------------------------------------------------------------------------

/// Marker for entities on the Main hub screen (despawned on exit Main).
#[derive(Component)]
pub(super) struct OnMainScreen;

/// Actions from hub buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WizardTowerButtonAction {
    ReturnToMenu,
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

/// Actions from study screen buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StudyButtonAction {
    Commit,
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

/// Target of a study allocation +/- button. Either a spell (for unlock
/// progress) or an insight bonus stat (for bonus upgrade progress).
#[derive(Debug, Clone, Copy)]
pub(super) enum AllocTarget {
    Spell(Spell),
    Bonus(crate::game::insight_bonuses::InsightBonusStat),
}

/// Marks a +/- button next to a study allocation slider. `delta` is in
/// insight units (positive to increase, negative to decrease).
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct StudyAllocAdjustButton {
    pub target: AllocTarget,
    pub delta: i32,
}

// ---------------------------------------------------------------------------
// Talent UI components
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Insight constellation components
// ---------------------------------------------------------------------------

/// Marks an insight bonus node in the constellation with its graph-space position.
#[derive(Component)]
pub(super) struct InsightBonusNode {
    pub stat: InsightBonusStat,
    pub graph_position: Vec2,
}

/// Marks the central anchor of the insight constellation.
#[derive(Component)]
pub(super) struct InsightConstellationAnchor;

/// Marks an edge segment in the insight constellation.
#[derive(Component)]
pub(super) struct InsightConstellationEdge {
    pub start: Vec2,
    pub end: Vec2,
}

/// Resource tracking the currently selected insight bonus for the detail panel.
/// Mutually exclusive with `SelectedStudySpell`.
#[derive(Resource, Default)]
pub(super) struct SelectedInsightBonus(pub Option<InsightBonusStat>);

/// Marks the slider track for an insight bonus allocation.
#[derive(Component)]
pub(super) struct InsightBonusSlider {
    pub stat: InsightBonusStat,
}

/// Marks the committed progress fill in an insight bonus slider.
#[derive(Component)]
pub(super) struct InsightBonusProgressFill;

/// Marks the pending allocation fill in an insight bonus slider.
#[derive(Component)]
pub(super) struct InsightBonusAllocationFill {
    pub stat: InsightBonusStat,
}

/// Marks the slider handle for an insight bonus allocation.
#[derive(Component)]
pub(super) struct InsightBonusSliderHandle {
    pub stat: InsightBonusStat,
    pub is_dragging: bool,
}

/// Text showing allocation progress for an insight bonus.
#[derive(Component)]
pub(super) struct InsightBonusAllocationText {
    pub stat: InsightBonusStat,
}

/// Marks a concentric rings material node on an insight bonus graph node.
#[derive(Component)]
pub(super) struct InsightBonusRings {
    pub stat: InsightBonusStat,
}

/// Marks a text label inside a graph node that should scale with zoom.
#[derive(Component)]
pub(super) struct GraphNodeLabel {
    pub base_size: f32,
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
