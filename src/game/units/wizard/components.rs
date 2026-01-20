use bevy::prelude::*;

/// Marker component for the wizard entity.
#[derive(Component)]
pub struct Wizard;

/// Mana component for the wizard.
///
/// Tracks current and maximum mana for spell casting.
#[derive(Component)]
pub struct Mana {
    /// Current mana amount.
    pub current: f32,
    /// Maximum mana capacity.
    pub max: f32,
}

impl Mana {
    /// Creates a new Mana component with the given maximum.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Returns true if there is enough mana for the cost.
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= cost
    }

    /// Consumes mana, returning true if successful.
    pub fn consume(&mut self, cost: f32) -> bool {
        if self.can_afford(cost) {
            self.current -= cost;
            true
        } else {
            false
        }
    }

    /// Regenerates mana, clamped to max.
    pub fn regenerate(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Returns mana as a percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

/// Mana regeneration component.
///
/// Defines how fast mana regenerates per second.
#[derive(Component)]
pub struct ManaRegen {
    /// Mana regenerated per second.
    pub rate: f32,
}

impl ManaRegen {
    /// Creates a new ManaRegen component.
    pub const fn new(rate: f32) -> Self {
        Self { rate }
    }
}
