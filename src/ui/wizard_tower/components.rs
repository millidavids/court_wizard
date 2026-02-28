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

/// Scrollable container for the spell research grid.
#[derive(Component)]
pub struct ScrollableResearchContainer;

/// Pending Insight allocation display in study header.
#[derive(Component)]
pub(super) struct PendingInsightDisplay;

/// Insight balance display in study header.
#[derive(Component)]
pub(super) struct StudyInsightDisplay;

// ---------------------------------------------------------------------------
// Allocation slider components (per-spell)
// ---------------------------------------------------------------------------

/// The clickable track area of a spell's allocation slider.
#[derive(Component)]
pub(super) struct AllocationSliderTrack {
    pub spell: Spell,
}

/// The filled portion of a spell's allocation slider.
#[derive(Component)]
pub(super) struct AllocationSliderFill {
    pub spell: Spell,
}

/// The draggable handle of a spell's allocation slider.
#[derive(Component)]
pub(super) struct AllocationSliderHandle {
    pub spell: Spell,
    pub is_dragging: bool,
}

/// Text showing "current+pending / total" for a spell.
#[derive(Component)]
pub(super) struct AllocationText {
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
