use std::collections::HashMap;

use super::super::save_cache::{load_unified_save, save_unified};
use super::super::save_structs::AchievementId;
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

/// Batch-set multiple insight bonus levels in a single load/save operation.
/// Used when insight has already been deducted via `spend_insight`.
pub(crate) fn set_insight_bonus_levels(updates: &[(&str, u8)]) {
    if updates.is_empty() {
        return;
    }
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    for &(id, level) in updates {
        save_file
            .player
            .insight_bonuses
            .insert(id.to_string(), level);
    }
    save_unified(&save_file);
}

/// Returns the research progress (insight invested) for a specific spell.
pub(crate) fn get_spell_research_progress(spell: Spell) -> u32 {
    let name = format!("{:?}", spell);
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
    let name = format!("{:?}", spell);
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
    let name = format!("{:?}", spell);
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
            .entry(format!("{:?}", spell))
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
    let name = format!("{:?}", spell);
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
    let name = format!("{:?}", spell);
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
