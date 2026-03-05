use bevy::prelude::*;

use super::brews::{BrewEffect, Recipe};

/// An active buff from a completed brew.
#[derive(Debug, Clone)]
pub struct ActiveBuff {
    pub effects: Vec<BrewEffect>,
    pub time_remaining: f32,
}

/// Resource tracking active cauldron buffs affecting the wizard.
#[derive(Resource, Debug, Clone, Default)]
pub struct CauldronBuffs {
    /// Currently active buffs with remaining durations.
    pub active_buffs: Vec<ActiveBuff>,
}

impl CauldronBuffs {
    /// Applies a recipe's effects as a new active buff.
    pub fn apply_recipe(&mut self, recipe: &Recipe) {
        self.active_buffs.push(ActiveBuff {
            effects: recipe.effects(),
            time_remaining: recipe.buff_duration(),
        });
    }

    /// Applies a recipe's effects with a potency multiplier (from Transmutation talent).
    /// Multiplier effects have their deviation from 1.0 scaled, flat effects are scaled directly.
    pub fn apply_recipe_with_potency(&mut self, recipe: &Recipe, potency: f32) {
        let effects: Vec<BrewEffect> = recipe
            .effects()
            .into_iter()
            .map(|effect| amplify_brew_effect(effect, potency))
            .collect();
        self.active_buffs.push(ActiveBuff {
            effects,
            time_remaining: recipe.buff_duration(),
        });
    }

    /// Ticks all active buff timers and removes expired ones.
    pub fn tick(&mut self, delta: f32) {
        self.active_buffs.retain_mut(|buff| {
            buff.time_remaining -= delta;
            buff.time_remaining > 0.0
        });
    }

    /// Returns true if any buff is currently active.
    pub fn has_active_buffs(&self) -> bool {
        !self.active_buffs.is_empty()
    }

    /// Resets all buffs (used on game cleanup).
    pub fn reset(&mut self) {
        self.active_buffs.clear();
    }

    /// Combined mana regeneration multiplier from all active buffs.
    pub fn mana_regen_multiplier(&self) -> f32 {
        self.compute_multiplier(|effect| match effect {
            BrewEffect::ManaRegenMultiplier(v) => Some(*v),
            _ => None,
        })
    }

    /// Combined spell power multiplier from all active buffs.
    pub fn spell_power_multiplier(&self) -> f32 {
        self.compute_multiplier(|effect| match effect {
            BrewEffect::SpellPowerMultiplier(v) => Some(*v),
            _ => None,
        })
    }

    /// Total defender heal per second from all active buffs.
    pub fn defender_heal_per_second(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::DefenderHealPerSecond(v) => Some(*v),
            _ => None,
        })
    }

    /// Combined cast speed multiplier from all active buffs.
    pub fn cast_speed_multiplier(&self) -> f32 {
        self.compute_multiplier(|effect| match effect {
            BrewEffect::CastSpeedMultiplier(v) => Some(*v),
            _ => None,
        })
    }

    /// Combined spell range multiplier from all active buffs.
    pub fn spell_range_multiplier(&self) -> f32 {
        self.compute_multiplier(|effect| match effect {
            BrewEffect::SpellRangeMultiplier(v) => Some(*v),
            _ => None,
        })
    }

    /// Total defender damage bonus from all active buffs.
    pub fn defender_damage_bonus(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::DefenderDamageBonus(v) => Some(*v),
            _ => None,
        })
    }

    /// Total damage resistance percentage for defenders from all active buffs.
    pub fn damage_resistance_percent(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::DamageResistancePercent(v) => Some(*v),
            _ => None,
        })
    }

    /// Total defender speed bonus from all active buffs.
    pub fn defender_speed_bonus(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::DefenderSpeedBonus(v) => Some(*v),
            _ => None,
        })
    }

    /// Total attacker slow percentage from all active buffs.
    pub fn attacker_slow_percent(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::AttackerSlowPercent(v) => Some(*v),
            _ => None,
        })
    }

    /// Total defender shield per second from all active buffs.
    pub fn defender_shield_per_second(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::DefenderShieldPerSecond(v) => Some(*v),
            _ => None,
        })
    }

    /// Combined max mana multiplier from all active buffs.
    pub fn max_mana_multiplier(&self) -> f32 {
        self.compute_multiplier(|effect| match effect {
            BrewEffect::MaxManaMultiplier(v) => Some(*v),
            _ => None,
        })
    }

    /// Combined attack speed multiplier from all active buffs.
    pub fn attack_speed_multiplier(&self) -> f32 {
        self.compute_multiplier(|effect| match effect {
            BrewEffect::AttackSpeedMultiplier(v) => Some(*v),
            _ => None,
        })
    }

    /// Total effectiveness bonus from all active buffs.
    pub fn effectiveness_bonus(&self) -> f32 {
        self.sum_flat_effect(|effect| match effect {
            BrewEffect::EffectivenessBonus(v) => Some(*v),
            _ => None,
        })
    }

    /// Computes a combined multiplier by scanning all active buff effects.
    fn compute_multiplier(&self, extract: impl Fn(&BrewEffect) -> Option<f32>) -> f32 {
        let mut result = 1.0;
        for buff in &self.active_buffs {
            for effect in &buff.effects {
                if let Some(value) = extract(effect) {
                    result *= value;
                }
            }
        }
        result
    }

    /// Sums a flat effect value across all active buffs.
    fn sum_flat_effect(&self, extract: impl Fn(&BrewEffect) -> Option<f32>) -> f32 {
        let mut total = 0.0;
        for buff in &self.active_buffs {
            for effect in &buff.effects {
                if let Some(value) = extract(effect) {
                    total += value;
                }
            }
        }
        total
    }
}

/// Amplifies a brew effect by a potency multiplier.
/// Multiplier effects (base 1.0) scale the deviation: 1.0 + (v - 1.0) * potency
/// Flat effects (base 0.0) scale the value directly: v * potency
fn amplify_brew_effect(effect: BrewEffect, potency: f32) -> BrewEffect {
    match effect {
        // Multiplier effects: amplify deviation from 1.0
        BrewEffect::ManaRegenMultiplier(v) => {
            BrewEffect::ManaRegenMultiplier(1.0 + (v - 1.0) * potency)
        }
        BrewEffect::SpellPowerMultiplier(v) => {
            BrewEffect::SpellPowerMultiplier(1.0 + (v - 1.0) * potency)
        }
        BrewEffect::CastSpeedMultiplier(v) => {
            BrewEffect::CastSpeedMultiplier(1.0 + (v - 1.0) * potency)
        }
        BrewEffect::SpellRangeMultiplier(v) => {
            BrewEffect::SpellRangeMultiplier(1.0 + (v - 1.0) * potency)
        }
        BrewEffect::MaxManaMultiplier(v) => {
            BrewEffect::MaxManaMultiplier(1.0 + (v - 1.0) * potency)
        }
        BrewEffect::AttackSpeedMultiplier(v) => {
            BrewEffect::AttackSpeedMultiplier(1.0 + (v - 1.0) * potency)
        }
        BrewEffect::BuffDurationMultiplier(v) => {
            BrewEffect::BuffDurationMultiplier(1.0 + (v - 1.0) * potency)
        }
        // Flat effects: scale directly
        BrewEffect::DefenderHealPerSecond(v) => BrewEffect::DefenderHealPerSecond(v * potency),
        BrewEffect::DefenderDamageBonus(v) => BrewEffect::DefenderDamageBonus(v * potency),
        BrewEffect::DamageResistancePercent(v) => BrewEffect::DamageResistancePercent(v * potency),
        BrewEffect::DefenderSpeedBonus(v) => BrewEffect::DefenderSpeedBonus(v * potency),
        BrewEffect::AttackerSlowPercent(v) => BrewEffect::AttackerSlowPercent(v * potency),
        BrewEffect::DefenderShieldPerSecond(v) => BrewEffect::DefenderShieldPerSecond(v * potency),
        BrewEffect::EffectivenessBonus(v) => BrewEffect::EffectivenessBonus(v * potency),
    }
}

/// Stores the cauldron sprite sheet texture handle.
#[derive(Resource)]
pub struct CauldronAssets {
    pub sprite_texture: Handle<Image>,
}
