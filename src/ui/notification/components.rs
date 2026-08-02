use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::wizard::components::Spell;

/// A queued notification entry — wizard unlock, ingredient, spell research, talent tier, combo, or settings toast.
pub(crate) enum NotificationEntry {
    WizardUnlocked(WizardType),
    IngredientCollected(Ingredient),
    SpellResearched(Spell),
    TalentTierUnlocked {
        spell: Spell,
        tier: u8,
    },
    ComboDiscovered {
        name: &'static str,
        description: &'static str,
    },
    Toast {
        message: &'static str,
    },
}

/// Marker for the notification root entity.
#[derive(Component)]
pub(super) struct Notification;

/// Queue of notifications waiting to be displayed.
#[derive(Resource, Default)]
pub(crate) struct NotificationQueue {
    pub queue: Vec<NotificationEntry>,
}

impl NotificationQueue {
    /// Add an entry to the queue.
    pub fn push(&mut self, entry: NotificationEntry) {
        self.queue.push(entry);
    }

    /// Get the next entry to display (if any).
    pub fn pop(&mut self) -> Option<NotificationEntry> {
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

/// Timer that controls notification display and fade-out.
#[derive(Component)]
pub(super) struct NotificationTimer {
    pub elapsed: f32,
    pub display_duration: f32,
    pub fade_duration: f32,
    /// The background's alpha at spawn time. The fade multiplies this by the
    /// current opacity each frame; without a fixed base the alpha would compound
    /// itself frame over frame and collapse long before the fade finishes.
    pub base_bg_alpha: f32,
}

impl NotificationTimer {
    pub fn new(display_duration: f32, fade_duration: f32, base_bg_alpha: f32) -> Self {
        Self {
            elapsed: 0.0,
            display_duration,
            fade_duration,
            base_bg_alpha,
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

    /// Returns true when the notification should be despawned.
    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.total_duration()
    }
}
