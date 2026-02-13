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
        let mut total = 0.0;
        for buff in &self.active_buffs {
            for effect in &buff.effects {
                if let BrewEffect::DefenderHealPerSecond(v) = effect {
                    total += v;
                }
            }
        }
        total
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
}

/// Stores the cauldron sprite sheet texture handle.
#[derive(Resource)]
pub struct CauldronAssets {
    pub sprite_texture: Handle<Image>,
}
