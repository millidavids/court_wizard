use bevy::prelude::*;

/// Effectiveness coefficient applied to movement speed and attack damage.
///
/// Dynamically calculated based on:
/// - Number of allies in melee range (positive effect)
/// - Number of enemies in melee range (negative effect)
/// - Spell-based modifiers (future feature)
///
/// Units with allies nearby become more effective; units surrounded by enemies
/// become less effective. This creates strategic depth and rewards good positioning.
#[derive(Component, Clone, Copy)]
pub struct Effectiveness {
    /// Current effectiveness multiplier (applied to speed and damage).
    pub current: f32,
    /// Base effectiveness before any modifiers (always 1.0).
    pub base: f32,
    /// Bonus from spell effects (e.g. raise-the-dead revenant damage) and the
    /// additive poison penalty. Shared/contended field — do NOT use it for the
    /// cauldron (see `cauldron_spell_bonus`).
    pub spell_bonus: f32,
    /// Effectiveness bonus from the Alchemist's cauldron brews. Kept separate
    /// from `spell_bonus` so the cauldron's overwrite-style write never clobbers
    /// the poison status effect's additive bookkeeping — which is what lets the
    /// cauldron effectiveness buff replicate to a guest's army in multiplayer.
    pub cauldron_spell_bonus: f32,
    /// Co-op difficulty bonus: a flat boost applied to ATTACKERS while a co-op
    /// guest is connected (own field so it composes with the others and clears
    /// the instant the guest disconnects). Always 0 in single-player and versus.
    pub coop_difficulty_bonus: f32,
}

impl Effectiveness {
    /// Creates a new Effectiveness component with default values.
    pub const fn new() -> Self {
        Self {
            current: 1.0,
            base: 1.0,
            spell_bonus: 0.0,
            cauldron_spell_bonus: 0.0,
            coop_difficulty_bonus: 0.0,
        }
    }

    /// Recalculates effectiveness based on proximity modifiers and spell bonuses.
    ///
    /// Formula: current = clamp(base + proximity_modifier + spell_bonus + cauldron_spell_bonus, MIN, MAX)
    ///
    /// # Arguments
    /// * `ally_count` - Number of allies in melee range
    /// * `enemy_count` - Number of enemies in melee range
    pub fn recalculate(&mut self, ally_count: i32, enemy_count: i32) {
        use crate::game::constants::{
            EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT,
            EFFECTIVENESS_MAX, EFFECTIVENESS_MIN,
        };

        let proximity_modifier = (ally_count as f32 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT)
            + (enemy_count as f32 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT);

        self.current = (self.base
            + proximity_modifier
            + self.spell_bonus
            + self.cauldron_spell_bonus
            + self.coop_difficulty_bonus)
            .clamp(EFFECTIVENESS_MIN, EFFECTIVENESS_MAX);
    }

    /// Returns the current effectiveness multiplier.
    pub fn multiplier(&self) -> f32 {
        self.current
    }
}

impl Default for Effectiveness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::constants::{
        EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT, EFFECTIVENESS_MAX,
        EFFECTIVENESS_MIN,
    };

    #[test]
    fn test_effectiveness_base_values() {
        let eff = Effectiveness::new();
        assert_eq!(eff.current, 1.0);
        assert_eq!(eff.base, 1.0);
        assert_eq!(eff.spell_bonus, 0.0);
        assert_eq!(eff.cauldron_spell_bonus, 0.0);
    }

    #[test]
    fn test_effectiveness_default() {
        let eff = Effectiveness::default();
        assert_eq!(eff.current, 1.0);
        assert_eq!(eff.base, 1.0);
        assert_eq!(eff.spell_bonus, 0.0);
        assert_eq!(eff.cauldron_spell_bonus, 0.0);
    }

    #[test]
    fn test_effectiveness_ally_bonus() {
        let mut eff = Effectiveness::new();
        eff.recalculate(3, 0); // 3 allies, 0 enemies
        assert_eq!(eff.current, 1.0 + 3.0 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT);
    }

    #[test]
    fn test_effectiveness_enemy_penalty() {
        let mut eff = Effectiveness::new();
        eff.recalculate(0, 2); // 0 allies, 2 enemies
        assert_eq!(
            eff.current,
            1.0 + 2.0 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT
        );
    }

    #[test]
    fn test_effectiveness_mixed() {
        let mut eff = Effectiveness::new();
        eff.recalculate(2, 1); // 2 allies, 1 enemy
        let expected = 1.0
            + 2.0 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT
            + 1.0 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT;
        // Approximate: `recalculate` sums the terms in a different association
        // order than `expected`, so the float results differ in the last ulp.
        assert!((eff.current - expected).abs() < 1e-4);
    }

    #[test]
    fn test_effectiveness_clamping_min() {
        let mut eff = Effectiveness::new();
        eff.recalculate(0, 10); // Many enemies
        assert_eq!(eff.current, EFFECTIVENESS_MIN);
    }

    #[test]
    fn test_effectiveness_clamping_max() {
        let mut eff = Effectiveness::new();
        eff.recalculate(20, 0); // Many allies
        assert_eq!(eff.current, EFFECTIVENESS_MAX);
    }

    #[test]
    fn test_effectiveness_with_spell_bonus() {
        let mut eff = Effectiveness::new();
        eff.spell_bonus = 0.5;
        eff.recalculate(0, 0); // No proximity modifiers
        assert_eq!(eff.current, 1.0 + 0.5);
    }

    #[test]
    fn test_effectiveness_cauldron_composes_with_spell_bonus() {
        // The cauldron bonus and the (poison-shared) spell_bonus must STACK, not
        // clobber: a poisoned unit (negative spell_bonus) that's also under a
        // cauldron effectiveness brew should net the two.
        let mut eff = Effectiveness::new();
        eff.spell_bonus = -0.2; // e.g. a poison penalty
        eff.cauldron_spell_bonus = 0.5; // cauldron effectiveness brew
        eff.recalculate(0, 0);
        assert_eq!(eff.current, 1.0 - 0.2 + 0.5);
    }

    #[test]
    fn test_effectiveness_multiplier() {
        let mut eff = Effectiveness::new();
        eff.recalculate(2, 1);
        assert_eq!(eff.multiplier(), eff.current);
    }
}
