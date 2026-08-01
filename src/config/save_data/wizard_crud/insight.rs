use std::collections::HashMap;

use super::super::save_cache::{load_unified_save, save_unified};
use super::super::save_structs::{AchievementId, PlayerMetaProgress};
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Arcane Insight & Spell Research
// ---------------------------------------------------------------------------

/// Grant Arcane Insight to the player and persist immediately.
pub(crate) fn grant_insight(amount: u32) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.arcane_insight += amount;
    save_unified(&save_file);
}

/// Spend Arcane Insight. Returns true if the player had enough and it was deducted.
pub(crate) fn spend_insight(amount: u32) -> bool {
    let Some(mut save_file) = load_unified_save() else {
        return false;
    };
    if save_file.player.arcane_insight < amount {
        return false;
    }
    save_file.player.arcane_insight -= amount;
    save_unified(&save_file);
    true
}

/// Returns the player's current Arcane Insight balance.
pub(crate) fn get_insight() -> u32 {
    load_unified_save()
        .map(|s| s.player.arcane_insight)
        .unwrap_or(0)
}

/// Returns true if the given toggle modifier has been permanently unlocked.
pub(crate) fn is_toggle_unlocked(
    toggle: crate::game::game_mode::components::ToggleModifier,
) -> bool {
    let id = toggle.id();
    load_unified_save()
        .map(|s| s.player.unlocked_toggles.iter().any(|t| t == id))
        .unwrap_or(false)
}

/// Returns all permanently unlocked toggle modifier IDs.
pub(crate) fn get_unlocked_toggles() -> Vec<String> {
    load_unified_save()
        .map(|s| s.player.unlocked_toggles.clone())
        .unwrap_or_default()
}

/// Unlock a toggle modifier by spending Insight. Returns true if successful.
pub(crate) fn unlock_toggle(toggle: crate::game::game_mode::components::ToggleModifier) -> bool {
    let id = toggle.id().to_string();
    let cost = toggle.insight_cost();

    let Some(mut save_file) = load_unified_save() else {
        return false;
    };
    // Already unlocked
    if save_file.player.unlocked_toggles.iter().any(|t| t == &id) {
        return false;
    }
    // Not enough insight
    if save_file.player.arcane_insight < cost {
        return false;
    }
    save_file.player.arcane_insight -= cost;
    save_file.player.unlocked_toggles.push(id);
    save_unified(&save_file);
    true
}

/// Returns all insight bonus levels as a map of id → level.
pub(crate) fn get_all_insight_bonuses() -> HashMap<String, u8> {
    load_unified_save()
        .map(|s| s.player.insight_bonuses.clone())
        .unwrap_or_default()
}

/// Effective banked progress for a bonus stat: the stored raw progress, floored
/// at whatever the mirrored level implies. The floor migrates saves written
/// before partial progress was tracked, where only the level exists. Capped at
/// `total_cost` so tampered-with data can't produce an out-of-range value.
fn effective_bonus_progress(stored: u32, mirrored_level: u8) -> u32 {
    let implied = mirrored_level as u32 * InsightBonusStat::cost_per_level();
    stored.max(implied).min(InsightBonusStat::total_cost())
}

/// Insight banked toward one bonus stat, reading the source-of-truth map and
/// falling back to the legacy level mirror for saves that predate it.
fn banked_progress(player: &PlayerMetaProgress, id: &str) -> u32 {
    effective_bonus_progress(
        player.insight_bonus_progress.get(id).copied().unwrap_or(0),
        player.insight_bonuses.get(id).copied().unwrap_or(0),
    )
}

/// Write banked progress for a stat, keeping the derived level mirror in step.
/// Every writer of either map goes through here, so the two cannot drift.
pub(super) fn set_bonus_progress(player: &mut PlayerMetaProgress, id: &str, progress: u32) {
    let progress = progress.min(InsightBonusStat::total_cost());
    player
        .insight_bonus_progress
        .insert(id.to_string(), progress);
    player.insight_bonuses.insert(
        id.to_string(),
        InsightBonusStat::level_for_progress(progress),
    );
}

/// Returns the insight banked toward a bonus stat, including partial progress
/// below the next level threshold. Single source of truth for "how much has
/// been committed here" — allocation caps and the UI both read through this.
pub(crate) fn get_insight_bonus_progress(stat_id: &str) -> u32 {
    load_unified_save()
        .map(|s| banked_progress(&s.player, stat_id))
        .unwrap_or(0)
}

/// Banked progress for every bonus stat from a single save load. Prefer this
/// over per-stat reads in loops — each load deep-clones the whole save file.
pub(crate) fn get_all_insight_bonus_progress() -> HashMap<String, u32> {
    let Some(save_file) = load_unified_save() else {
        return HashMap::new();
    };
    InsightBonusStat::all()
        .iter()
        .map(|stat| {
            (
                stat.id().to_string(),
                banked_progress(&save_file.player, stat.id()),
            )
        })
        .collect()
}

/// Bank insight toward one or more bonus stats in a single load/save operation.
/// Used when insight has already been deducted via `spend_insight`.
///
/// Returns true if any stat crossed a whole-level threshold.
pub(crate) fn add_insight_bonus_progress(updates: &[(&str, u32)]) -> bool {
    let Some(mut save_file) = load_unified_save() else {
        return false;
    };

    let mut dirty = false;
    let mut leveled_up = false;
    for &(id, amount) in updates {
        if amount == 0 {
            continue;
        }
        let old = banked_progress(&save_file.player, id);
        let progress = old.saturating_add(amount);
        set_bonus_progress(&mut save_file.player, id, progress);

        leveled_up |= InsightBonusStat::level_for_progress(progress)
            > InsightBonusStat::level_for_progress(old);
        dirty = true;
    }

    if dirty {
        save_unified(&save_file);
    }
    leveled_up
}

/// Returns the research progress (insight invested) for a specific spell.
pub(crate) fn get_spell_research_progress(spell: Spell) -> u32 {
    let name = spell.save_key().to_string();
    load_unified_save()
        .and_then(|s| s.player.spell_research_progress.get(&name).copied())
        .unwrap_or(0)
}

/// Add research progress to a spell and persist. Also unlocks the spell if cost is met.
/// Returns true if the spell was newly unlocked by this progress.
pub(crate) fn add_spell_research_progress(spell: Spell, amount: u32) -> bool {
    let Some(mut save_file) = load_unified_save() else {
        return false;
    };
    let name = spell.save_key().to_string();
    let entry = save_file
        .player
        .spell_research_progress
        .entry(name.clone())
        .or_insert(0);
    *entry += amount;

    // Check if research is complete
    let cost = spell.research_cost();
    let newly_unlocked =
        if *entry >= cost && !save_file.player.unlocked_content.spells.contains(&name) {
            // Cap progress at cost
            *entry = cost;
            save_file.player.unlocked_content.spells.push(name);
            true
        } else {
            false
        };

    save_unified(&save_file);
    newly_unlocked
}

// ---------------------------------------------------------------------------
// Spell Talent Progress & Selections
// ---------------------------------------------------------------------------

/// Returns the talent progress for a specific spell.
pub(crate) fn get_spell_talent_progress(spell: Spell) -> u32 {
    let name = spell.save_key().to_string();
    load_unified_save()
        .and_then(|s| s.player.spell_talent_progress.get(&name).copied())
        .unwrap_or(0)
}

/// Apply many talent-progress increments at once, returning the pre-increment
/// value for each spell. One save load + one save write regardless of input size.
pub(crate) fn add_spell_talent_progress_batch(
    increments: &HashMap<Spell, u32>,
) -> HashMap<Spell, u32> {
    let mut prev_values = HashMap::new();
    let Some(mut save_file) = load_unified_save() else {
        return prev_values;
    };
    let mut dirty = false;
    for (&spell, &amount) in increments {
        if amount == 0 {
            continue;
        }
        let entry = save_file
            .player
            .spell_talent_progress
            .entry(spell.save_key().to_string())
            .or_insert(0);
        prev_values.insert(spell, *entry);
        *entry += amount;
        dirty = true;
    }
    if dirty {
        save_unified(&save_file);
    }
    prev_values
}

/// Returns the talent selections for a spell as [Option<u8>; 3].
pub(crate) fn get_spell_talent_selections(spell: Spell) -> [Option<u8>; 3] {
    let name = spell.save_key().to_string();
    let raw =
        load_unified_save().and_then(|s| s.player.spell_talent_selections.get(&name).cloned());

    match raw {
        Some(vec) => {
            let mut result = [None; 3];
            for (i, &val) in vec.iter().take(3).enumerate() {
                result[i] = if val >= 0 { Some(val as u8) } else { None };
            }
            result
        }
        None => [None; 3],
    }
}

/// Set a talent selection for a spell at a given tier and persist.
pub(crate) fn set_spell_talent_selection(spell: Spell, tier: usize, choice: Option<u8>) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    let name = spell.save_key().to_string();
    let entry = save_file
        .player
        .spell_talent_selections
        .entry(name)
        .or_insert_with(|| vec![-1, -1, -1]);

    // Ensure vec is at least 3 elements
    while entry.len() < 3 {
        entry.push(-1);
    }

    if tier < 3 {
        entry[tier] = match choice {
            Some(c) => c as i8,
            None => -1,
        };
    }

    save_unified(&save_file);
}

/// Grant one-time Insight bonus for an achievement. Returns the amount granted (0 if none).
pub(crate) fn grant_achievement_insight(id: AchievementId) -> u32 {
    let amount = id.insight_reward();
    if amount > 0 {
        grant_insight(amount);
    }
    amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_progress_prefers_stored_value() {
        assert_eq!(effective_bonus_progress(0, 0), 0);
        assert_eq!(effective_bonus_progress(400, 0), 400);
        assert_eq!(effective_bonus_progress(1700, 3), 1700);
    }

    #[test]
    fn effective_progress_falls_back_to_level_for_old_saves() {
        // Saves written before partial progress existed have only a level.
        assert_eq!(effective_bonus_progress(0, 3), 1500);
        assert_eq!(effective_bonus_progress(0, 5), 2500);
    }

    #[test]
    fn effective_progress_clamps_tampered_values() {
        assert_eq!(effective_bonus_progress(9999, 5), 2500);
        assert_eq!(effective_bonus_progress(0, 200), 2500);
    }

    /// The regression this fix exists for: two sub-threshold commits used to
    /// cost 550 insight and grant nothing. They must now accumulate and level.
    #[test]
    fn sub_threshold_insight_accumulates_across_commits() {
        let first = effective_bonus_progress(0, 0).saturating_add(250);
        assert_eq!(first, 250);
        assert_eq!(InsightBonusStat::level_for_progress(first), 0);

        let second = effective_bonus_progress(first, 0).saturating_add(300);
        assert_eq!(second, 550);
        assert_eq!(InsightBonusStat::level_for_progress(second), 1);
    }

    #[test]
    fn banking_onto_an_old_save_starts_from_the_mirrored_level() {
        assert_eq!(effective_bonus_progress(0, 3).saturating_add(300), 1800);
    }

    #[test]
    fn banking_saturates_rather_than_overflowing() {
        let progress = effective_bonus_progress(u32::MAX, 0).saturating_add(u32::MAX);
        assert_eq!(InsightBonusStat::level_for_progress(progress), 5);
    }
}
