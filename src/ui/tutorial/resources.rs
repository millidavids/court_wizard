//! Tutorial resources.

use bevy::prelude::*;

use super::definitions::TutorialId;

/// Resource tracking the currently active tutorial and step.
/// Inserted when a tutorial starts, removed when it completes or is skipped.
#[derive(Resource)]
pub(crate) struct ActiveTutorial {
    pub tutorial: TutorialId,
    pub step: usize,
    /// Whether gameplay was paused for this tutorial (InGame only).
    pub paused_gameplay: bool,
}

/// FIFO queue of tutorials waiting to play. When a tutorial completes,
/// the next id (if any) is popped and started immediately. Triggers that
/// fire while another tutorial is active append to this queue instead of
/// being silently dropped — so e.g. opening a tab on the very first visit
/// while the controller-menus primer is still running plays the tab
/// tutorial right after, not on the second visit.
#[derive(Resource, Default)]
pub(crate) struct PendingTutorials {
    pub queue: std::collections::VecDeque<TutorialId>,
}

/// Resource tracking which tutorials the player has completed.
/// Synced with save file on load/save.
#[derive(Resource, Default)]
pub(crate) struct TutorialProgress {
    pub completed: Vec<String>,
}

impl TutorialProgress {
    pub fn is_completed(&self, tutorial: &TutorialId) -> bool {
        self.completed.contains(&tutorial.id().to_string())
    }

    pub fn mark_completed(&mut self, tutorial: &TutorialId) {
        let id = tutorial.id().to_string();
        if !self.completed.contains(&id) {
            self.completed.push(id);
        }
    }

    pub fn reset(&mut self) {
        self.completed.clear();
    }
}
