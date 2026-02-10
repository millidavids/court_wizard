use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::units::wizard::components::Spell;

use super::progress::{keyed_hash, load_verified_progress, to_hex};
use super::resources::{
    ActiveSave, GameConfig, WizardType, deserialize_action_bar, serialize_action_bar,
};
use super::storage;

/// Maximum number of save slots.
pub const MAX_SAVE_SLOTS: usize = 3;

/// Per-save progress data that gets signed and stored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct SaveData {
    pub(crate) wizard_name: String,
    pub(crate) wizard_type: WizardType,
    pub(crate) current_level: u32,
    pub(crate) highest_level_achieved: u32,
    pub(crate) efficiency_ratios: HashMap<String, f32>,
    #[serde(
        default,
        serialize_with = "serialize_action_bar",
        deserialize_with = "deserialize_action_bar"
    )]
    pub(crate) action_bar_slots: [Option<Spell>; 5],
}

/// Signed save data container with data and its signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedSaveData {
    signature: String,
    data: SaveData,
}

/// Lightweight summary for displaying saves in menus.
#[derive(Debug, Clone)]
pub(crate) struct SaveSummary {
    pub(crate) slot: usize,
    pub(crate) wizard_name: String,
    pub(crate) wizard_type: WizardType,
    pub(crate) current_level: u32,
    #[allow(dead_code)]
    pub(crate) highest_level_achieved: u32,
}

/// Computes the signature for the given save data.
fn compute_save_signature(data: &SaveData) -> String {
    let canonical = toml::to_string(data).unwrap_or_default();
    let hash = keyed_hash(canonical.as_bytes());
    to_hex(hash)
}

/// Saves save data to a specific slot in localStorage (signed).
pub(crate) fn save_to_slot(slot: usize, data: &SaveData) {
    let signature = compute_save_signature(data);
    let signed = SignedSaveData {
        signature,
        data: data.clone(),
    };

    match toml::to_string_pretty(&signed) {
        Ok(toml_string) => {
            if let Err(e) = storage::save_slot(slot, &toml_string) {
                error!("Failed to save to slot {}: {}", slot, e);
            }
        }
        Err(e) => {
            error!("Failed to serialize save data for slot {}: {}", slot, e);
        }
    }
}

/// Loads and verifies save data from a specific slot.
/// Returns None if missing, tampered, or invalid.
pub(crate) fn load_from_slot(slot: usize) -> Option<SaveData> {
    let contents = storage::load_slot(slot).ok()?;
    let signed: SignedSaveData = toml::from_str(&contents).ok()?;

    let expected = compute_save_signature(&signed.data);
    if expected == signed.signature {
        Some(signed.data)
    } else {
        warn!(
            "Save slot {} signature mismatch — save has been tampered with",
            slot
        );
        None
    }
}

/// Loads summaries of all occupied save slots.
pub(crate) fn load_all_summaries() -> Vec<SaveSummary> {
    let mut summaries = Vec::new();
    for slot in 0..MAX_SAVE_SLOTS {
        if let Some(data) = load_from_slot(slot) {
            summaries.push(SaveSummary {
                slot,
                wizard_name: data.wizard_name,
                wizard_type: data.wizard_type,
                current_level: data.current_level,
                highest_level_achieved: data.highest_level_achieved,
            });
        }
    }
    summaries
}

/// Finds the first available (empty) save slot.
pub(crate) fn find_next_available_slot() -> Option<usize> {
    (0..MAX_SAVE_SLOTS).find(|&slot| !storage::slot_exists(slot))
}

/// Checks if a wizard name is already used by an existing save.
pub(crate) fn is_name_taken(name: &str) -> bool {
    load_all_summaries()
        .iter()
        .any(|s| s.wizard_name.eq_ignore_ascii_case(name))
}

/// Deletes a save slot from localStorage.
pub(crate) fn delete_slot(slot: usize) {
    if let Err(e) = storage::delete_slot(slot) {
        error!("Failed to delete save slot {}: {}", slot, e);
    }
}

/// Loads save data from a slot into the active GameConfig and sets ActiveSave.
/// Returns true if successful.
pub(crate) fn load_save_into_config(
    slot: usize,
    config: &mut GameConfig,
    active_save: &mut ActiveSave,
) -> bool {
    if let Some(data) = load_from_slot(slot) {
        config.wizard_name = data.wizard_name;
        config.wizard_type = data.wizard_type;
        config.current_level = data.current_level;
        config.highest_level_achieved = data.highest_level_achieved;
        config.efficiency_ratios = data.efficiency_ratios;
        config.action_bar_slots = data.action_bar_slots;
        active_save.0 = Some(slot);
        true
    } else {
        false
    }
}

/// Saves the current GameConfig progress to the active save slot.
pub(crate) fn save_config_to_active_slot(config: &GameConfig, active_save: &ActiveSave) {
    if let Some(slot) = active_save.0 {
        let data = SaveData {
            wizard_name: config.wizard_name.clone(),
            wizard_type: config.wizard_type,
            current_level: config.current_level,
            highest_level_achieved: config.highest_level_achieved,
            efficiency_ratios: config.efficiency_ratios.clone(),
            action_bar_slots: config.action_bar_slots,
        };
        save_to_slot(slot, &data);
    }
}

/// Migrates legacy single-save progress to slot 0 if no save slots exist.
pub(crate) fn migrate_legacy_progress(config: &GameConfig) {
    // Only migrate if old progress exists and no new saves exist
    if let Some(old_progress) = load_verified_progress() {
        if !storage::slot_exists(0) && !storage::slot_exists(1) && !storage::slot_exists(2) {
            let save = SaveData {
                wizard_name: "Wizard".to_string(),
                wizard_type: WizardType::RuneCaster,
                current_level: old_progress.current_level,
                highest_level_achieved: old_progress.highest_level_achieved,
                efficiency_ratios: old_progress.efficiency_ratios,
                action_bar_slots: config.action_bar_slots,
            };
            save_to_slot(0, &save);
            let _ = storage::delete_progress();
            info!("Migrated legacy progress to save slot 0");
        }
    }
}
