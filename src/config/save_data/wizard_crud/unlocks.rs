use super::super::save_cache::{
    flush_save_cache, load_unified_save, new_unified_save, save_unified,
};
use super::super::save_structs::{AchievementId, UnlockedContent};
use crate::config::resources::WizardType;
use crate::game::units::UnitType;
use crate::game::units::wizard::components::Spell;

/// Unlock an achievement and persist immediately.
/// Returns true if the achievement was newly unlocked.
pub(crate) fn unlock_achievement(id: AchievementId) -> bool {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    let id_str = id.id().to_string();
    if save_file.player.unlocked_achievements.contains(&id_str) {
        return false;
    }
    save_file.player.unlocked_achievements.push(id_str);
    save_unified(&save_file);
    true
}

/// Unlock a wizard type and persist immediately.
/// Returns true if the wizard type was newly unlocked.
pub(crate) fn unlock_wizard_type(wizard_type: WizardType) -> bool {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    let name = wizard_type.save_key().to_string();
    if save_file
        .player
        .unlocked_content
        .wizard_types
        .contains(&name)
    {
        return false;
    }
    save_file.player.unlocked_content.wizard_types.push(name);
    save_unified(&save_file);
    true
}

/// Unlock an ingredient and persist immediately.
/// Returns true if the ingredient was newly unlocked.
pub(crate) fn unlock_ingredient(ingredient: crate::game::cauldron::brews::Ingredient) -> bool {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    let name = ingredient.save_key().to_string();
    if save_file
        .player
        .unlocked_content
        .ingredients
        .contains(&name)
    {
        return false;
    }
    save_file.player.unlocked_content.ingredients.push(name);
    save_unified(&save_file);
    true
}

/// Unlock a unit type and persist immediately.
/// Returns true if the unit was newly unlocked.
pub(crate) fn unlock_unit(unit_type: UnitType) -> bool {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    let name = unit_type.save_key().to_string();
    if save_file.player.unlocked_content.units.contains(&name) {
        return false;
    }
    save_file.player.unlocked_content.units.push(name);
    save_unified(&save_file);
    true
}

/// Dev-only: unlock every piece of content for testing. Performs the whole
/// unlock in a single load → mutate → save → flush. Bound to the orange
/// "Unlock Everything" debug button on the Gameplay settings tab (revealed by F2).
#[cfg(debug_assertions)]
pub(crate) fn unlock_everything_for_testing() {
    use crate::game::game_mode::components::ToggleModifier;
    use crate::game::insight_bonuses::InsightBonusStat;
    use crate::game::units::wizard::talents::constants::tier_thresholds;

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    let player = &mut save_file.player;

    // Spells: unlock all, fill research progress to completion (so the Study shows
    // them researched), and push talent progress past the top tier threshold so
    // every talent tier is selectable (this does not auto-pick a branch).
    player.unlocked_content.spells = UnlockedContent::all_spells();
    for spell in Spell::all() {
        let name = spell.save_key().to_string();
        let cost = spell.research_cost();
        if cost > 0 {
            player.spell_research_progress.insert(name.clone(), cost);
        }
        // Unlock all talent tiers without lowering a player's higher banked
        // progress: keep the max of existing progress and the top-tier threshold.
        let top_tier = tier_thresholds(*spell)[2];
        player
            .spell_talent_progress
            .entry(name)
            .and_modify(|v| *v = (*v).max(top_tier))
            .or_insert(top_tier);
    }

    // Ingredients (PhilosophersStone is intentionally excluded from Ingredient::all()
    // — it is Alchemist-only and never appears in the unlock list), plus wizard
    // types, units, and brew combos.
    player.unlocked_content.ingredients = UnlockedContent::all_ingredients();
    player.unlocked_content.wizard_types = WizardType::all()
        .iter()
        .map(|w| w.save_key().to_string())
        .collect();
    player.unlocked_content.units = UnitType::all()
        .iter()
        .map(|u| u.save_key().to_string())
        .collect();
    player.unlocked_content.combos = crate::game::cauldron::brews::constants::all_combo_names()
        .map(|name| name.to_string())
        .collect();

    // Endless-mode toggle modifiers (stable string IDs), maxed Insight stat
    // bonuses, and a large Arcane Insight balance.
    player.unlocked_toggles = ToggleModifier::all()
        .iter()
        .map(|t| t.id().to_string())
        .collect();
    for stat in InsightBonusStat::all() {
        super::insight::set_bonus_progress(player, stat.id(), InsightBonusStat::total_cost());
    }
    player.arcane_insight = player.arcane_insight.max(99_999);

    save_unified(&save_file);
    flush_save_cache();
}

/// Clear all progress and start a fresh save, keeping the previous save as a
/// single rollback backup at `<save>.cleared` (overwriting any prior backup).
pub(crate) fn clear_progress() {
    flush_save_cache();
    if let Err(e) = crate::config::storage::move_unified_save_to_cleared_backup() {
        bevy::log::warn!("Failed to back up save before clearing progress: {e}");
    }
    save_unified(&new_unified_save());
    flush_save_cache();
}
