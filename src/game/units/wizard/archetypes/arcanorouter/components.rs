use bevy::prelude::*;

/// Component that tracks stat bonuses applied by the Arcanorouter archetype.
///
/// These bonuses are applied as multipliers to the Wizard's base stats.
/// Values are stored as percentages (100.0 = 1.0x multiplier).
#[derive(Component, Debug, Clone)]
pub struct ArcanoRouterBonuses {
    pub range_bonus: f32,
    pub mana_cost_bonus: f32,
    pub spell_power_bonus: f32,
    pub cast_speed_bonus: f32,
}

impl Default for ArcanoRouterBonuses {
    fn default() -> Self {
        Self {
            range_bonus: 100.0,
            mana_cost_bonus: 100.0,
            spell_power_bonus: 100.0,
            cast_speed_bonus: 100.0,
        }
    }
}

impl ArcanoRouterBonuses {
    /// Converts a bonus percentage to a multiplier (100% = 1.0x)
    pub fn get_range_multiplier(&self) -> f32 {
        self.range_bonus / 100.0
    }

    /// Higher mana allocation = CHEAPER spells. Reciprocal of the allocation %
    /// (default 100 → 1.0×; 200 → 0.5×; 50 → 2.0×), so raising the bar reduces
    /// cost and it never reaches zero.
    pub fn get_mana_cost_multiplier(&self) -> f32 {
        100.0 / self.mana_cost_bonus
    }

    pub fn get_spell_power_multiplier(&self) -> f32 {
        self.spell_power_bonus / 100.0
    }

    pub fn get_cast_speed_multiplier(&self) -> f32 {
        self.cast_speed_bonus / 100.0
    }
}
