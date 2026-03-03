use std::collections::HashMap;

use bevy::prelude::*;

use crate::config::save_data;
use crate::game::units::wizard::components::Spell;

/// Tracks which talents the player has selected for each spell.
/// Loaded from save data on entering relevant states.
#[derive(Resource, Default)]
pub(crate) struct ActiveTalents {
    pub selections: HashMap<Spell, [Option<u8>; 3]>,
}

impl ActiveTalents {
    /// Load talent selections from save data.
    pub fn from_save() -> Self {
        let mut selections = HashMap::new();
        for spell in Spell::all() {
            let sel = save_data::get_spell_talent_selections(*spell);
            if sel.iter().any(|s| s.is_some()) {
                selections.insert(*spell, sel);
            }
        }
        Self { selections }
    }

    /// Get the selected talent choice for a spell at a given tier.
    pub fn get_selection(&self, spell: Spell, tier: usize) -> Option<u8> {
        self.selections
            .get(&spell)
            .and_then(|sels| sels.get(tier).copied().flatten())
    }

    /// Set a talent selection for a spell at a given tier.
    #[allow(dead_code)]
    pub fn set_selection(&mut self, spell: Spell, tier: usize, choice: Option<u8>) {
        let entry = self.selections.entry(spell).or_insert([None; 3]);
        if let Some(slot) = entry.get_mut(tier) {
            *slot = choice;
        }
        save_data::set_spell_talent_selection(spell, tier, choice);
    }

    /// Check if a specific talent is active for a spell.
    #[allow(dead_code)]
    pub fn has_talent(&self, spell: Spell, tier: usize, choice: u8) -> bool {
        self.get_selection(spell, tier) == Some(choice)
    }
}

/// Accumulates talent progress during a single battle.
/// Flushed to save data when the battle ends.
#[derive(Resource, Default)]
pub(crate) struct BattleTalentProgress {
    pub progress: HashMap<Spell, u32>,
}

impl BattleTalentProgress {
    /// Increment progress for a spell by the given amount.
    pub fn increment(&mut self, spell: Spell, amount: u32) {
        *self.progress.entry(spell).or_insert(0) += amount;
    }

    /// Flush all accumulated progress to save data.
    pub fn flush_to_save(&self) {
        for (&spell, &amount) in &self.progress {
            if amount > 0 {
                save_data::add_spell_talent_progress(spell, amount);
            }
        }
    }
}
