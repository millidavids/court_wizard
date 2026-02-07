use bevy::prelude::*;

use super::brews::Recipe;

/// Marker component for the cauldron entity.
#[derive(Component)]
pub struct Cauldron;

/// Tracks the current state of the cauldron.
#[derive(Debug, Clone, Component, Default)]
pub enum CauldronState {
    /// Ready to start a brew.
    #[default]
    Idle,
    /// Currently brewing — wizard cannot cast spells.
    Brewing {
        recipe: Recipe,
        elapsed: f32,
        duration: f32,
    },
    /// Short cooldown between brews.
    #[allow(dead_code)]
    Cooldown { remaining: f32 },
}

impl CauldronState {
    /// Returns true if the cauldron is currently brewing.
    pub fn is_brewing(&self) -> bool {
        matches!(self, Self::Brewing { .. })
    }

    /// Returns true if the cauldron is idle and ready for a new brew.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns the recipe currently being brewed, if any.
    #[allow(dead_code)]
    pub fn active_recipe(&self) -> Option<&Recipe> {
        match self {
            Self::Brewing { recipe, .. } => Some(recipe),
            _ => None,
        }
    }

    /// Cancels the current brew, returning to idle.
    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    /// Starts brewing the given recipe.
    pub fn start_brewing(&mut self, recipe: Recipe, duration: f32) {
        *self = Self::Brewing {
            recipe,
            elapsed: 0.0,
            duration,
        };
    }

    /// Advances the brew timer. Returns Some(recipe) if brewing just completed.
    pub fn tick(&mut self, delta: f32) -> Option<Recipe> {
        match self {
            Self::Brewing {
                elapsed, duration, ..
            } => {
                *elapsed += delta;
                if *elapsed >= *duration {
                    // Take the recipe out before transitioning to Idle
                    let old = std::mem::take(self);
                    if let CauldronState::Brewing { recipe, .. } = old {
                        Some(recipe)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Self::Cooldown { remaining } => {
                *remaining -= delta;
                if *remaining <= 0.0 {
                    *self = Self::Idle;
                }
                None
            }
            Self::Idle => None,
        }
    }

    /// Returns brew progress as a percentage (0.0 to 1.0).
    #[allow(dead_code)]
    pub fn progress(&self) -> f32 {
        match self {
            Self::Brewing {
                elapsed, duration, ..
            } => {
                if *duration > 0.0 {
                    (elapsed / duration).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}
