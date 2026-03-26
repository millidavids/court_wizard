use std::collections::HashMap;

use bevy::prelude::*;

use crate::config::save_data;
use crate::game::units::wizard::components::Spell;

use super::constants::tier_thresholds;

/// Tracks which talents the player has selected for each spell.
/// Loaded from save data on entering relevant states.
#[derive(Resource, Default)]
pub(crate) struct ActiveTalents {
    pub selections: HashMap<Spell, [Option<u8>; 3]>,
}

impl ActiveTalents {
    /// Load talent selections from save data, clearing any selections
    /// in tiers the player hasn't actually unlocked yet.
    pub fn from_save() -> Self {
        // Load save file once for validation; only write back if we fix anything
        let mut save_file = save_data::load_unified_save();
        let mut save_dirty = false;

        let mut selections = HashMap::new();
        for spell in Spell::all() {
            let mut sel = save_data::get_spell_talent_selections(*spell);
            let progress = save_data::get_spell_talent_progress(*spell);
            let thresholds = tier_thresholds(*spell);

            // Clear selections in locked tiers
            for (tier, slot) in sel.iter_mut().enumerate() {
                if slot.is_some() && progress < thresholds[tier] {
                    warn!(
                        "Clearing invalid talent selection for {:?} tier {} \
                         (progress {} < threshold {})",
                        spell, tier, progress, thresholds[tier]
                    );
                    *slot = None;

                    // Fix in the loaded save file (batched write at end)
                    if let Some(ref mut sf) = save_file {
                        let name = format!("{:?}", spell);
                        if let Some(entry) = sf.player.spell_talent_selections.get_mut(&name) {
                            if tier < entry.len() {
                                entry[tier] = -1;
                                save_dirty = true;
                            }
                        }
                    }
                }
            }

            if sel.iter().any(|s| s.is_some()) {
                selections.insert(*spell, sel);
            }
        }

        // Single write if any selections were cleaned up
        if save_dirty {
            if let Some(ref sf) = save_file {
                save_data::save_unified(sf);
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
