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
                    if let Some(ref mut sf) = save_file
                        && let Some(entry) = sf
                            .player
                            .spell_talent_selections
                            .get_mut(&format!("{:?}", spell))
                        && tier < entry.len()
                    {
                        entry[tier] = -1;
                        save_dirty = true;
                    }
                }
            }

            if sel.iter().any(|s| s.is_some()) {
                selections.insert(*spell, sel);
            }
        }

        // Single write if any selections were cleaned up
        if save_dirty && let Some(ref sf) = save_file {
            save_data::save_unified(sf);
        }

        Self { selections }
    }

    /// Get the selected talent choice for a spell at a given tier.
    pub fn get_selection(&self, spell: Spell, tier: usize) -> Option<u8> {
        self.selections
            .get(&spell)
            .and_then(|sels| sels.get(tier).copied().flatten())
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

    /// Flush all accumulated progress to save data and return any tier thresholds
    /// that were crossed by this flush.
    pub fn flush_to_save(&self) -> Vec<(Spell, u8)> {
        let prev_values = save_data::add_spell_talent_progress_batch(&self.progress);
        if prev_values.is_empty() && !self.progress.is_empty() {
            warn!(
                "flush_to_save: save I/O returned no prior values for {} spells; \
                 tier-crossing notifications will be suppressed this battle.",
                self.progress.len()
            );
        }
        let mut crossed = Vec::new();
        for (&spell, &prev) in &prev_values {
            let amount = self.progress.get(&spell).copied().unwrap_or(0);
            let new_total = prev + amount;
            let thresholds = tier_thresholds(spell);
            for (i, &threshold) in thresholds.iter().enumerate() {
                if prev < threshold && new_total >= threshold {
                    crossed.push((spell, i as u8));
                }
            }
        }
        crossed
    }
}
