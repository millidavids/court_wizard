use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Shared (both Main and Study screens)
// ---------------------------------------------------------------------------

/// Marker for entities that should be despawned when exiting WizardTower state entirely.
#[derive(Component)]
pub(crate) struct OnWizardTowerScreen;

/// Marker for the header and content rows (toggled by F3 debug).
#[cfg(debug_assertions)]
#[derive(Component)]
pub(crate) struct WizardTowerUiContent;

/// Marker for the arcane rune background MaterialNode.
#[derive(Component)]
pub(crate) struct ArcaneRuneBackground;

/// Orbiting spell name text around the arcane rune circles.
#[derive(Component)]
pub(crate) struct ArcaneRuneText {
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
pub(crate) struct OnMainScreen;

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
pub(crate) struct TimeTravelSection;

/// Clickable level entry in the time travel list.
#[derive(Component)]
pub(crate) struct TimeTravelLevelButton(pub u32);

/// Text showing the currently selected time travel level.
#[derive(Component)]
pub(crate) struct TimeTravelSelectedDisplay;

/// Tracks which level is selected in the time travel list.
#[derive(Resource, Default)]
pub(crate) struct SelectedTimeTravelLevel(pub Option<u32>);

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
pub(crate) struct PendingInsightDisplay;

/// Insight balance display in study header.
#[derive(Component)]
pub(crate) struct StudyInsightDisplay;

// ---------------------------------------------------------------------------
// Study detail panel display components
// ---------------------------------------------------------------------------

/// Marks the floating detail panel entity.
#[derive(Component)]
pub(crate) struct StudyDetailPanel;

/// Marks a detail panel's allocation slider track.
#[derive(Component)]
pub(crate) struct StudyAllocationSlider {
    pub spell: Spell,
}

/// Marks the committed progress fill (non-draggable floor) in the unified slider.
#[derive(Component)]
pub(crate) struct StudyProgressFill;

/// Marks the pending allocation fill (draggable region) in the unified slider.
#[derive(Component)]
pub(crate) struct StudyAllocationFill {
    pub spell: Spell,
}

/// Marks a detail panel's allocation slider handle.
#[derive(Component)]
pub(crate) struct StudyAllocationHandle {
    pub spell: Spell,
    pub is_dragging: bool,
}

/// Text showing allocation progress in the detail panel.
#[derive(Component)]
pub(crate) struct StudyAllocationText {
    pub spell: Spell,
}

/// Target of a study allocation +/- button. Either a spell (for unlock
/// progress) or an insight bonus stat (for bonus upgrade progress).
#[derive(Debug, Clone, Copy)]
pub(crate) enum AllocTarget {
    Spell(Spell),
    Bonus(crate::game::insight_bonuses::InsightBonusStat),
}

/// Marks a +/- button next to a study allocation slider. `delta` is in
/// insight units (positive to increase, negative to decrease).
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct StudyAllocAdjustButton {
    pub(crate) target: AllocTarget,
    pub delta: i32,
}

// ---------------------------------------------------------------------------
// Talent UI components
// ---------------------------------------------------------------------------

/// Marker for a clickable talent card.
#[derive(Component)]
pub(crate) struct TalentCard {
    pub spell: Spell,
    pub tier: u8,
    pub choice: u8,
}

/// Marker for the talent progress bar fill.
#[derive(Component)]
pub(crate) struct TalentProgressBarFill {
    #[allow(dead_code)]
    pub spell: Spell,
}

/// Marker for the talent description text area.
#[derive(Component)]
pub(crate) struct TalentDescriptionText;
