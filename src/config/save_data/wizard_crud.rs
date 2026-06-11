use std::collections::HashMap;

use bevy::prelude::*;

use super::save_cache::{
    current_timestamp, flush_save_cache, load_unified_save, new_unified_save, save_unified,
};
use super::save_structs::{
    AchievementId, EndlessLevelBest, RogueliteData, RogueliteRun, SavedLevelTerrain,
    SavedRogueliteRun, SavedTrampling, UnlockedContent, WizardSave,
};
use crate::config::resources::{ActiveSave, GameConfig, WizardType};
use crate::game::units::UnitType;
use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Unlocks
// ---------------------------------------------------------------------------

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
    let name = format!("{:?}", wizard_type);
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
    let name = format!("{:?}", ingredient);
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
    let name = format!("{:?}", unit_type);
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
        let name = format!("{:?}", spell);
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
        .map(|w| format!("{:?}", w))
        .collect();
    player.unlocked_content.units = UnitType::all().iter().map(|u| format!("{:?}", u)).collect();
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
        player
            .insight_bonuses
            .insert(stat.id().to_string(), InsightBonusStat::max_level());
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
        warn!("Failed to back up save before clearing progress: {e}");
    }
    save_unified(&new_unified_save());
    flush_save_cache();
}

// ---------------------------------------------------------------------------
// Wizard CRUD
// ---------------------------------------------------------------------------

/// Get the saved wizard for a specific wizard type (if one exists).
pub(crate) fn get_wizard_by_type(wizard_type: WizardType) -> Option<WizardSave> {
    let save_file = load_unified_save()?;
    save_file
        .wizards
        .into_iter()
        .find(|w| w.wizard_type == wizard_type)
}

/// Validates action bar slots against currently unlocked spells.
/// Clears any slots containing locked spells.
fn validate_action_bar_slots(action_bar_slots: &mut [Option<Spell>; 5]) {
    let save_file = load_unified_save();
    let unlocked_spells: Vec<String> = save_file
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    for slot in action_bar_slots.iter_mut() {
        if let Some(spell) = slot {
            let debug_name = format!("{:?}", spell);
            if !unlocked_spells.contains(&debug_name) {
                *slot = None; // Clear locked spell from action bar
            }
        }
    }
}

/// Load the wizard for a given type into GameConfig and set it as active.
/// Returns true if a save existed and was loaded.
pub(crate) fn load_wizard_type_into_config(
    wizard_type: WizardType,
    config: &mut GameConfig,
    active_save: &mut ActiveSave,
) -> bool {
    let Some(wizard) = get_wizard_by_type(wizard_type) else {
        return false;
    };

    config.wizard_type = wizard.wizard_type;
    config.current_level = wizard.current_level;
    config.highest_level_achieved = wizard.highest_level_achieved;
    config.efficiency_ratios = wizard.efficiency_ratios.clone();
    config.action_bar_slots = wizard.action_bar_slots;
    config.saved_walls = wizard.saved_walls.clone();
    config.saved_crystals = wizard.saved_crystals.clone();
    config.saved_flora = wizard.saved_flora.clone();
    config.saved_trampling = wizard.saved_trampling.clone();

    // Validate that all action bar slots contain unlocked spells
    validate_action_bar_slots(&mut config.action_bar_slots);

    active_save.0 = Some(wizard.id.clone());
    true
}

/// Loads an existing wizard of the given type into config, or creates a fresh
/// wizard record if none exists and then loads that. On return, `config` and
/// `active_save` reflect the wizard's current state (fully reset for new ones).
pub(crate) fn load_or_create_wizard(
    wizard_type: WizardType,
    config: &mut GameConfig,
    active_save: &mut ActiveSave,
) {
    if load_wizard_type_into_config(wizard_type, config, active_save) {
        return;
    }
    create_wizard(wizard_type);
    // Reload now that the wizard exists so config fully syncs (including fields
    // like saved_walls/crystals/flora/trampling that would otherwise leak from a
    // previously-active wizard).
    load_wizard_type_into_config(wizard_type, config, active_save);
}

/// Create a new wizard and add it to the save file.
/// Returns the new wizard's ID.
pub(crate) fn create_wizard(wizard_type: WizardType) -> String {
    use super::super::save_encoding::generate_id;

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    let wizard = WizardSave {
        id: generate_id(),
        wizard_type,
        current_level: 1,
        highest_level_achieved: 1,
        created_at: current_timestamp(),
        last_played_at: current_timestamp(),
        efficiency_ratios: HashMap::new(),
        action_bar_slots: {
            let [a, b, c, d] = Spell::default_unlocked();
            [Some(a), Some(b), Some(c), Some(d), None]
        },
        saved_walls: Vec::new(),
        saved_crystals: Vec::new(),
        saved_flora: Vec::new(),
        saved_trampling: SavedTrampling::default(),
        roguelite: RogueliteData::default(),
        endless_best_stats: HashMap::new(),
        terrain_per_level: HashMap::new(),
    };

    let id = wizard.id.clone();
    save_file.wizards.push(wizard);
    save_unified(&save_file);
    id
}

/// Save the current GameConfig back to the active wizard in the unified save.
///
/// When `is_roguelite` is true, only action bar and timestamp are persisted —
/// level progress, walls, crystals, and efficiency stay untouched so the
/// wizard's Endless progress is never corrupted by a Roguelite run.
pub(crate) fn save_config_to_active_wizard(
    config: &GameConfig,
    active_save: &ActiveSave,
    is_roguelite: bool,
) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.wizard_type = config.wizard_type;
        wizard.action_bar_slots = config.action_bar_slots;
        wizard.last_played_at = current_timestamp();

        if !is_roguelite {
            wizard.current_level = config.current_level;
            wizard.highest_level_achieved = config.highest_level_achieved;
            wizard.efficiency_ratios = config.efficiency_ratios.clone();
            wizard.saved_walls = config.saved_walls.clone();
            wizard.saved_crystals = config.saved_crystals.clone();
            wizard.saved_flora = config.saved_flora.clone();
            wizard.saved_trampling = config.saved_trampling.clone();
        }
    }

    save_file.metadata.last_active_wizard_id = Some(wizard_id.clone());
    save_unified(&save_file);
}

/// Saves the current terrain state as a per-level snapshot for Endless time travel.
/// Called on victory in Endless mode (non-time-travel).
pub(crate) fn save_level_terrain(active_save: &ActiveSave, level: u32, config: &GameConfig) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.terrain_per_level.insert(
            level.to_string(),
            SavedLevelTerrain {
                trees: config.saved_trees.clone(),
                ponds: config.saved_ponds.clone(),
                bushes: config.saved_bushes.clone(),
                boulders: config.saved_boulders.clone(),
                walls: config.saved_walls.clone(),
                crystals: config.saved_crystals.clone(),
                flora: config.saved_flora.clone(),
            },
        );
    }

    save_unified(&save_file);
}

/// Loads terrain for a specific level from the per-level terrain snapshots.
/// Used when time traveling in Endless mode. Loads terrain from level-1
/// (the end of the previous level = start of this level).
/// Returns true if terrain was found and loaded into config.
pub(crate) fn load_level_terrain_into_config(
    active_save: &ActiveSave,
    level: u32,
    config: &mut GameConfig,
) -> bool {
    let Some(wizard_id) = &active_save.0 else {
        return false;
    };

    let save_file = load_unified_save();
    let Some(save_file) = save_file else {
        return false;
    };

    let Some(wizard) = save_file.wizards.iter().find(|w| &w.id == wizard_id) else {
        return false;
    };

    // Load terrain from the end of level-1 (= start of this level).
    // For level 1, there's no previous level, so clear terrain (it will be regenerated).
    if level <= 1 {
        config.saved_trees.clear();
        config.saved_ponds.clear();
        config.saved_bushes.clear();
        config.saved_boulders.clear();
        config.saved_walls.clear();
        config.saved_crystals.clear();
        config.saved_flora.clear();
        return true;
    }

    let prev_key = (level - 1).to_string();
    let Some(terrain) = wizard.terrain_per_level.get(&prev_key) else {
        return false;
    };

    config.saved_trees = terrain.trees.clone();
    config.saved_ponds = terrain.ponds.clone();
    config.saved_bushes = terrain.bushes.clone();
    config.saved_boulders = terrain.boulders.clone();
    config.saved_walls = terrain.walls.clone();
    config.saved_crystals = terrain.crystals.clone();
    config.saved_flora = terrain.flora.clone();
    true
}

// ---------------------------------------------------------------------------
// Roguelite run ops
// ---------------------------------------------------------------------------

/// Save a completed roguelite run to the wizard's run history.
/// Caps at MAX_ROGUELITE_RUN_HISTORY entries (FIFO).
pub(crate) fn save_roguelite_run(active_save: &ActiveSave, run: RogueliteRun) {
    use crate::game::game_mode::components::MAX_ROGUELITE_RUN_HISTORY;

    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.roguelite.run_history.push(run);
        // Trim oldest unsaved runs when over limit (single pass)
        let unsaved_count = wizard
            .roguelite
            .run_history
            .iter()
            .filter(|r| !r.saved)
            .count();
        if unsaved_count > MAX_ROGUELITE_RUN_HISTORY {
            let excess = unsaved_count - MAX_ROGUELITE_RUN_HISTORY;
            let mut removed = 0;
            wizard.roguelite.run_history.retain(|r| {
                if !r.saved && removed < excess {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    save_unified(&save_file);
}

/// Saves the current in-progress roguelite run to disk so it can be resumed later.
/// Called when returning to the wizard tower between levels.
pub(crate) fn save_current_roguelite_run(
    active_save: &ActiveSave,
    run: &crate::game::game_mode::components::RogueliteRunState,
    config: &crate::config::GameConfig,
    modifiers: Option<&crate::game::game_mode::components::RogueliteModifiers>,
    toggles: Option<&crate::game::game_mode::components::ActiveToggles>,
    seed: Option<u64>,
) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.roguelite.current_run = Some(SavedRogueliteRun {
            started_at: run.started_at,
            current_level: config.current_level,
            wizard_type: config.wizard_type,
            level_stats: run.level_stats.clone(),
            modifiers: modifiers.cloned(),
            seed,
            active_toggles: toggles.map(|t| t.to_ids()).unwrap_or_default(),
            accessibility_assists: config.has_accessibility_assists(),
        });
    }

    save_unified(&save_file);
}

/// Clears the current in-progress roguelite run from disk.
/// Called when the run ends (victory, explicit abandon, or exit-to-menu mid-level).
pub(crate) fn clear_current_roguelite_run(active_save: &ActiveSave) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.roguelite.current_run = None;
    }

    save_unified(&save_file);
}

/// Loads the current in-progress roguelite run from disk, if one exists.
pub(crate) fn load_current_roguelite_run(active_save: &ActiveSave) -> Option<SavedRogueliteRun> {
    let wizard_id = active_save.0.as_ref()?;
    let save_file = load_unified_save()?;
    let wizard = save_file.wizards.iter().find(|w| &w.id == wizard_id)?;
    wizard.roguelite.current_run.clone()
}

/// Toggle the saved status of a roguelite run identified by its `started_at` timestamp.
pub(crate) fn toggle_roguelite_run_saved(started_at: u64) {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    for wizard in &mut save_file.wizards {
        if let Some(run) = wizard
            .roguelite
            .run_history
            .iter_mut()
            .find(|r| r.started_at == started_at)
        {
            run.saved = !run.saved;
            break;
        }
    }

    save_unified(&save_file);
}

// ---------------------------------------------------------------------------
// Endless best stats
// ---------------------------------------------------------------------------

/// Returns the best stats for a specific endless level, if any have been recorded.
pub(crate) fn get_endless_best_stats(level: u32) -> Option<EndlessLevelBest> {
    let save_file = load_unified_save()?;
    let key = level.to_string();
    // Search all wizards for best stats at this level (return the best across wizards)
    save_file
        .wizards
        .iter()
        .filter_map(|w| w.endless_best_stats.get(&key).cloned())
        .max_by(|a, b| {
            a.best_efficiency
                .partial_cmp(&b.best_efficiency)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Update the best stats for an endless level if the current efficiency beats the stored best.
///
/// `coop` tags the entry when a co-op partner was connected; `coop_peer_name` is
/// their Steam display name (if known).
pub(crate) fn update_endless_best_stats(
    active_save: &ActiveSave,
    stats: &crate::game::game_mode::components::LevelRunStats,
    coop: bool,
    coop_peer_name: Option<String>,
) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        let key = stats.level.to_string();
        let should_update = wizard
            .endless_best_stats
            .get(&key)
            .is_none_or(|existing| stats.efficiency > existing.best_efficiency);

        if should_update {
            wizard.endless_best_stats.insert(
                key,
                EndlessLevelBest {
                    best_efficiency: stats.efficiency,
                    attackers_killed: stats.attackers_killed,
                    undead_killed: stats.undead_killed,
                    defenders_lost: stats.defenders_lost,
                    elapsed_time: stats.elapsed_time,
                    played_coop: coop,
                    coop_peer_name,
                },
            );
        }
    }

    save_unified(&save_file);
}

/// Records a co-op GUEST's endless result against their OWN wizard's save,
/// matched BY WIZARD TYPE (not the active save, which after a co-op match points
/// at a possibly-different wizard). Tags the entry as co-op and applies the
/// endless CONTIGUITY rule: the guest's frontier advances ONLY when this co-op
/// level is exactly their next level — joining a higher level and winning records
/// the result but does NOT skip them ahead. No-op (safe) if the guest has no save
/// for that wizard type, so it can never create or corrupt an unrelated save.
/// Records a co-op GUEST's level-end against their OWN save in a SINGLE load+save:
/// lifetime counters (games played, kill totals, and — on victory — levels
/// completed) plus, for an endless victory, the per-wizard endless best-stats with
/// the contiguity rule (frontier advances only when this is exactly their next
/// level). The per-wizard part is matched BY WIZARD TYPE and is a no-op if the
/// guest has no save for that wizard, so it can never corrupt an unrelated save.
/// `efficiency` is clamped to `[0, 1]`: a full wipe also kills the King's Guard,
/// which can push raw defender losses past the initial army size and make the raw
/// formula go negative.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_coop_guest_level_end(
    guest_wizard: crate::config::WizardType,
    is_endless: bool,
    victory: bool,
    level: u32,
    defenders_killed: u32,
    attackers_killed: u32,
    undead_killed: u32,
    efficiency: f32,
    elapsed_time: f32,
    coop_peer_name: Option<String>,
) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };

    // Lifetime counters (player-wide, every co-op mode, win or lose) — mirrors the
    // host's `send_battle_ended` accounting so shared activity counts for both.
    save_file.player.total_games_played += 1;
    save_file.player.total_defenders_killed += defenders_killed;
    save_file.player.total_attackers_killed += attackers_killed;
    save_file.player.total_undead_killed += undead_killed;
    if victory {
        save_file.player.total_levels_completed += 1;
    }

    // Per-wizard endless progression (endless victories only).
    if is_endless
        && victory
        && let Some(wizard) = save_file
            .wizards
            .iter_mut()
            .find(|w| w.wizard_type == guest_wizard)
    {
        let efficiency = efficiency.clamp(0.0, 1.0);
        let key = level.to_string();
        let should_update = wizard
            .endless_best_stats
            .get(&key)
            .is_none_or(|existing| efficiency > existing.best_efficiency);
        if should_update {
            wizard.endless_best_stats.insert(
                key,
                EndlessLevelBest {
                    best_efficiency: efficiency,
                    attackers_killed,
                    undead_killed,
                    defenders_lost: defenders_killed,
                    elapsed_time,
                    played_coop: true,
                    coop_peer_name,
                },
            );
        }
        // Contiguity: advance the guest's own frontier only if they cleared
        // their OWN next level.
        if level == wizard.current_level {
            wizard.current_level = level + 1;
            if wizard.highest_level_achieved < wizard.current_level {
                wizard.highest_level_achieved = wizard.current_level;
            }
        }
    }

    save_unified(&save_file);
}

// ---------------------------------------------------------------------------
// Meta-progression counters
// ---------------------------------------------------------------------------

/// Increment meta-progression counters on victory.
pub(crate) fn increment_levels_completed() {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_levels_completed += 1;
    save_unified(&save_file);
}

/// Increment total games played counter.
pub(crate) fn increment_games_played() {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_games_played += 1;
    save_unified(&save_file);
}

/// Accumulate per-battle kill stats into lifetime totals.
pub(crate) fn accumulate_kill_stats(defenders: u32, attackers: u32, undead: u32) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_defenders_killed += defenders;
    save_file.player.total_attackers_killed += attackers;
    save_file.player.total_undead_killed += undead;
    save_unified(&save_file);
}

/// Returns the total number of levels completed (victories) across all time.
pub(crate) fn get_total_levels_completed() -> u32 {
    load_unified_save()
        .map(|s| s.player.total_levels_completed)
        .unwrap_or(0)
}

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
