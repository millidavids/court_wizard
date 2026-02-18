use bevy::prelude::*;

use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::wizard::components::Spell;

/// A queued popup entry — either an achievement, ingredient, or spell research.
pub(super) enum PopupEntry {
    Achievement(AchievementId),
    IngredientCollected(Ingredient),
    SpellResearched(Spell),
}

/// Marker for the popup root entity.
#[derive(Component)]
pub(super) struct AchievementPopup;

/// Queue of popups waiting to be displayed.
#[derive(Resource, Default)]
pub(super) struct PopupQueue {
    pub queue: Vec<PopupEntry>,
}

impl PopupQueue {
    /// Add an entry to the queue.
    pub fn push(&mut self, entry: PopupEntry) {
        self.queue.push(entry);
    }

    /// Get the next entry to display (if any).
    pub fn pop(&mut self) -> Option<PopupEntry> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Timer that controls popup display and fade-out.
#[derive(Component)]
pub(super) struct AchievementPopupTimer {
    pub elapsed: f32,
    pub display_duration: f32,
    pub fade_duration: f32,
}

impl AchievementPopupTimer {
    pub fn new(display_duration: f32, fade_duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            display_duration,
            fade_duration,
        }
    }

    /// Total lifetime before despawn.
    pub fn total_duration(&self) -> f32 {
        self.display_duration + self.fade_duration
    }

    /// Returns the current opacity (1.0 during display, fading to 0.0).
    pub fn opacity(&self) -> f32 {
        if self.elapsed <= self.display_duration {
            1.0
        } else {
            let fade_elapsed = self.elapsed - self.display_duration;
            (1.0 - fade_elapsed / self.fade_duration).max(0.0)
        }
    }

    /// Returns true when the popup should be despawned.
    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.total_duration()
    }
}
