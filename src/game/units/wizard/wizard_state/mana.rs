use bevy::prelude::*;

/// Mana component for the wizard.
///
/// Tracks current and maximum mana for spell casting.
#[derive(Component)]
pub struct Mana {
    /// Current mana amount.
    pub current: f32,
    /// Maximum mana capacity.
    pub max: f32,
    /// Per-wizard spell-cost multiplier, mirrored from `Wizard.mana_cost_multiplier`
    /// by `sync_mana_cost_multiplier`. Applied to EVERY `consume`/`can_afford` so
    /// the Arcanorouter's mana routing, the BoringOleMage discount, and insight
    /// bonuses affect ALL spells (not just the few that read the Wizard field).
    /// 1.0 = no change.
    pub cost_multiplier: f32,
}

impl Mana {
    /// Creates a new Mana component with the given maximum.
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            cost_multiplier: 1.0,
        }
    }

    /// The cost actually deducted for a base `cost`, after the wizard multiplier.
    fn effective_cost(&self, cost: f32) -> f32 {
        cost * self.cost_multiplier
    }

    /// Returns true if there is enough mana for the cost (after the multiplier).
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= self.effective_cost(cost)
    }

    /// Consumes mana, returning true if successful.
    pub fn consume(&mut self, cost: f32) -> bool {
        if self.can_afford(cost) {
            self.current -= self.effective_cost(cost);
            true
        } else {
            false
        }
    }

    /// True if there is enough mana for an ALREADY-multiplied (raw) cost.
    /// Pair with [`Mana::consume_raw`] for spells that compute their own combined
    /// multiplier (e.g. a channeled spell stacking a talent discount with the
    /// Arcanorouter dial, clamped so the total never exceeds 50% off).
    pub fn can_afford_raw(&self, cost: f32) -> bool {
        self.current >= cost
    }

    /// Consumes an ALREADY-multiplied (raw) cost, bypassing `cost_multiplier` so
    /// the caller can apply a combined/clamped multiplier itself.
    pub fn consume_raw(&mut self, cost: f32) -> bool {
        if self.current >= cost {
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
