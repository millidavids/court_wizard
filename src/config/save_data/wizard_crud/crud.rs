use std::collections::HashMap;

use super::super::save_cache::{
    current_timestamp, load_unified_save, new_unified_save, save_unified,
};
use super::super::save_structs::{RogueliteData, SavedLevelTerrain, SavedTrampling, WizardSave};
use crate::config::resources::{ActiveSave, GameConfig, WizardType};
use crate::game::units::wizard::components::Spell;

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
            let debug_name = spell.save_key().to_string();
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
    use super::super::super::save_encoding::generate_id;

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
