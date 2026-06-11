use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::save_encoding::generate_id;
use super::save_cache::{
    current_timestamp, from_base64, new_unified_save, save_unified, to_base64,
};
use super::save_structs::{RogueliteData, SavedTrampling, WizardSave};
use crate::config::progress::{keyed_hash, load_verified_progress};
use crate::config::resources::{
    GameConfig, WizardType, deserialize_action_bar, serialize_action_bar,
};
use crate::config::storage;
use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Legacy migration
// ---------------------------------------------------------------------------

/// Maximum number of old save slots (for migration only).
const LEGACY_MAX_SAVE_SLOTS: usize = 3;

/// Old per-save progress data (for migration only).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacySaveData {
    wizard_name: String,
    wizard_type: WizardType,
    current_level: u32,
    highest_level_achieved: u32,
    #[serde(default)]
    efficiency_ratios: HashMap<String, f32>,
    #[serde(
        default,
        serialize_with = "serialize_action_bar",
        deserialize_with = "deserialize_action_bar"
    )]
    action_bar_slots: [Option<Spell>; 5],
}

/// XOR cipher using old slot-based key.
fn legacy_obfuscate(data: &[u8], slot: usize) -> Vec<u8> {
    let seed = format!("save_slot_{}", slot);
    let key_hash = keyed_hash(seed.as_bytes());
    let key_bytes = key_hash.to_le_bytes();

    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
        .collect()
}

/// Load a save from the old slot-based system.
fn load_legacy_slot(slot: usize) -> Option<LegacySaveData> {
    let encoded = storage::load_slot(slot).ok()?;
    let obfuscated = from_base64(&encoded)?;
    let deobfuscated = legacy_obfuscate(&obfuscated, slot);
    let toml_string = String::from_utf8(deobfuscated).ok()?;

    match toml::from_str::<LegacySaveData>(&toml_string) {
        Ok(data) => Some(data),
        Err(e) => {
            warn!("Failed to parse legacy save slot {}: {}", slot, e);
            None
        }
    }
}

/// Migrate legacy single-save progress (very old format) into a legacy slot.
fn migrate_very_old_progress(config: &GameConfig) {
    if let Some(old_progress) = load_verified_progress()
        && !storage::slot_exists(0)
        && !storage::slot_exists(1)
        && !storage::slot_exists(2)
    {
        // Create a legacy-format save in slot 0 so the main migration picks it up
        let save = LegacySaveData {
            wizard_name: "Wizard".to_string(),
            wizard_type: WizardType::RuneCaster,
            current_level: old_progress.current_level,
            highest_level_achieved: old_progress.highest_level_achieved,
            efficiency_ratios: old_progress.efficiency_ratios,
            action_bar_slots: config.action_bar_slots,
        };
        if let Ok(toml_string) = toml::to_string_pretty(&save) {
            let seed = "save_slot_0".to_string();
            let key_hash = keyed_hash(seed.as_bytes());
            let key_bytes = key_hash.to_le_bytes();
            let obfuscated: Vec<u8> = toml_string
                .as_bytes()
                .iter()
                .enumerate()
                .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
                .collect();
            let encoded = to_base64(&obfuscated);
            let _ = storage::save_slot(0, &encoded);
        }
        let _ = storage::delete_progress();
        info!("Migrated very old progress to legacy slot 0 for unified migration");
    }
}

/// Migrate all legacy save slots into the new unified save file.
/// Called at startup if no unified save exists.
/// If multiple legacy saves have the same wizard type, keeps the one with the highest level.
pub(crate) fn migrate_legacy_saves(config: &GameConfig) {
    // If unified save already exists, nothing to do
    if storage::unified_save_exists() {
        return;
    }

    // First handle the very old single-save format
    migrate_very_old_progress(config);

    // Collect all old slot-based saves
    let old_saves: Vec<(usize, LegacySaveData)> = (0..LEGACY_MAX_SAVE_SLOTS)
        .filter_map(|slot| load_legacy_slot(slot).map(|data| (slot, data)))
        .collect();

    if old_saves.is_empty() {
        return;
    }

    info!(
        "Migrating {} legacy save slot(s) to unified save file",
        old_saves.len()
    );

    let mut unified = new_unified_save();
    let now = current_timestamp();

    // Collect wizards, deduplicating by type (keep highest level)
    let mut best_by_type: HashMap<WizardType, WizardSave> = HashMap::new();

    for (_slot, old_data) in &old_saves {
        let wizard = WizardSave {
            id: generate_id(),
            wizard_type: old_data.wizard_type,
            current_level: old_data.current_level,
            highest_level_achieved: old_data.highest_level_achieved,
            created_at: now,
            last_played_at: now,
            efficiency_ratios: old_data.efficiency_ratios.clone(),
            action_bar_slots: old_data.action_bar_slots,
            saved_walls: Vec::new(),
            saved_crystals: Vec::new(),
            saved_flora: Vec::new(),
            saved_trampling: SavedTrampling::default(),
            roguelite: RogueliteData::default(),
            endless_best_stats: HashMap::new(),
            terrain_per_level: HashMap::new(),
        };

        let dominated = best_by_type
            .get(&wizard.wizard_type)
            .is_some_and(|existing| {
                existing.highest_level_achieved >= wizard.highest_level_achieved
            });

        if !dominated {
            best_by_type.insert(wizard.wizard_type, wizard);
        }
    }

    unified.wizards = best_by_type.into_values().collect();

    // Set initial meta-progression from migrated data
    unified.player.total_levels_completed = unified
        .wizards
        .iter()
        .map(|w| w.highest_level_achieved.saturating_sub(1))
        .sum();
    unified.player.total_games_played = unified.wizards.len() as u32;

    // Set last active to the highest-level wizard
    if let Some(best) = unified
        .wizards
        .iter()
        .max_by_key(|w| w.highest_level_achieved)
    {
        unified.metadata.last_active_wizard_id = Some(best.id.clone());
    }

    save_unified(&unified);

    // Clean up old slots
    for slot in 0..LEGACY_MAX_SAVE_SLOTS {
        let _ = storage::delete_slot(slot);
    }

    info!("Legacy save migration complete");
}
