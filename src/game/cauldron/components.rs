use bevy::prelude::*;

use super::brews::Brew;

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
        brew: Brew,
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

    /// Returns the brew currently being brewed, if any.
    pub fn active_brew(&self) -> Option<Brew> {
        match self {
            Self::Brewing { brew, .. } => Some(*brew),
            _ => None,
        }
    }

    /// Cancels the current brew, returning to idle.
    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    /// Starts brewing the given brew.
    pub fn start_brewing(&mut self, brew: Brew, duration: f32) {
        *self = Self::Brewing {
            brew,
            elapsed: 0.0,
            duration,
        };
    }

    /// Advances the brew timer. Returns Some(brew) if brewing just completed.
    pub fn tick(&mut self, delta: f32) -> Option<Brew> {
        match self {
            Self::Brewing {
                brew,
                elapsed,
                duration,
            } => {
                *elapsed += delta;
                if *elapsed >= *duration {
                    let completed_brew = *brew;
                    *self = Self::Idle;
                    Some(completed_brew)
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
