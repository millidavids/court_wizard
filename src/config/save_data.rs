use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::units::wizard::components::Spell;

use super::progress::{keyed_hash, load_verified_progress};
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

/// Simple XOR cipher for obfuscating save data.
/// Uses a key derived from the slot number and a secret constant.
fn obfuscate(data: &[u8], slot: usize) -> Vec<u8> {
    // Generate a keystream from the slot number
    let seed = format!("save_slot_{}", slot);
    let key_hash = keyed_hash(seed.as_bytes());
    let key_bytes = key_hash.to_le_bytes();

    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
        .collect()
}

/// Deobfuscate is the same as obfuscate (XOR is symmetric).
fn deobfuscate(data: &[u8], slot: usize) -> Vec<u8> {
    obfuscate(data, slot)
}

/// Convert bytes to base64 for storage.
fn to_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[(b3 & 0x3f) as usize] as char
        } else {
            '='
        });
    }

    result
}

/// Convert base64 back to bytes.
fn from_base64(s: &str) -> Option<Vec<u8>> {
    let chars: Vec<u8> = s.bytes().collect();
    let mut result = Vec::new();

    for chunk in chars.chunks(4) {
        if chunk.len() < 4 {
            break;
        }

        let decode = |c: u8| -> Option<u8> {
            match c {
                b'A'..=b'Z' => Some(c - b'A'),
                b'a'..=b'z' => Some(c - b'a' + 26),
                b'0'..=b'9' => Some(c - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                b'=' => Some(0),
                _ => None,
            }
        };

        let b1 = decode(chunk[0])?;
        let b2 = decode(chunk[1])?;
        let b3 = decode(chunk[2])?;
        let b4 = decode(chunk[3])?;

        result.push((b1 << 2) | (b2 >> 4));
        if chunk[2] != b'=' {
            result.push((b2 << 4) | (b3 >> 2));
        }
        if chunk[3] != b'=' {
            result.push((b3 << 6) | b4);
        }
    }

    Some(result)
}

/// Saves save data to a specific slot in localStorage (obfuscated).
pub(crate) fn save_to_slot(slot: usize, data: &SaveData) {
    match toml::to_string_pretty(data) {
        Ok(toml_string) => {
            // Obfuscate the TOML data
            let obfuscated = obfuscate(toml_string.as_bytes(), slot);
            let encoded = to_base64(&obfuscated);

            if let Err(e) = storage::save_slot(slot, &encoded) {
                error!("Failed to save to slot {}: {}", slot, e);
            }
        }
        Err(e) => {
            error!("Failed to serialize save data for slot {}: {}", slot, e);
        }
    }
}

/// Loads save data from a specific slot (deobfuscated).
/// Returns None if missing or invalid.
pub(crate) fn load_from_slot(slot: usize) -> Option<SaveData> {
    let encoded = storage::load_slot(slot).ok()?;

    // Try to decode from base64 and deobfuscate
    let obfuscated = from_base64(&encoded)?;
    let deobfuscated = deobfuscate(&obfuscated, slot);
    let toml_string = String::from_utf8(deobfuscated).ok()?;

    // Parse the TOML
    match toml::from_str::<SaveData>(&toml_string) {
        Ok(data) => Some(data),
        Err(e) => {
            warn!("Failed to parse save slot {}: {}", slot, e);
            None
        }
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
    if let Some(old_progress) = load_verified_progress()
        && !storage::slot_exists(0)
        && !storage::slot_exists(1)
        && !storage::slot_exists(2)
    {
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
