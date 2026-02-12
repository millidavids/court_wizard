use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::units::wizard::components::Spell;

use super::progress::{keyed_hash, load_verified_progress};
use super::resources::{
    ActiveSave, GameConfig, WizardType, deserialize_action_bar, serialize_action_bar,
};
use super::storage;

// ---------------------------------------------------------------------------
// Unified save file structures
// ---------------------------------------------------------------------------

/// Unified save file containing all wizards and player meta-progression.
/// Stored as a single entry in localStorage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UnifiedSaveFile {
    pub(crate) metadata: SaveMetadata,
    pub(crate) player: PlayerMetaProgress,
    #[serde(default)]
    pub(crate) wizards: Vec<WizardSave>,
}

/// Save file metadata for versioning and tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SaveMetadata {
    pub(crate) version: u32,
    pub(crate) last_active_wizard_id: Option<String>,
}

/// Player-level meta-progression (account-wide, not per-wizard).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PlayerMetaProgress {
    pub(crate) total_levels_completed: u32,
    pub(crate) total_games_played: u32,
}

/// Per-wizard save data. Exactly one per wizard type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WizardSave {
    pub(crate) id: String,
    pub(crate) wizard_type: WizardType,
    pub(crate) current_level: u32,
    pub(crate) highest_level_achieved: u32,
    pub(crate) created_at: u64,
    pub(crate) last_played_at: u64,
    #[serde(default)]
    pub(crate) efficiency_ratios: HashMap<String, f32>,
    #[serde(
        default,
        serialize_with = "serialize_action_bar",
        deserialize_with = "deserialize_action_bar"
    )]
    pub(crate) action_bar_slots: [Option<Spell>; 5],
}

// ---------------------------------------------------------------------------
// Obfuscation helpers
// ---------------------------------------------------------------------------

/// Simple XOR cipher for obfuscating save data.
fn obfuscate(data: &[u8]) -> Vec<u8> {
    let seed = b"unified_save_v2";
    let key_hash = keyed_hash(seed);
    let key_bytes = key_hash.to_le_bytes();

    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
        .collect()
}

/// Deobfuscate is the same as obfuscate (XOR is symmetric).
fn deobfuscate(data: &[u8]) -> Vec<u8> {
    obfuscate(data)
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

// ---------------------------------------------------------------------------
// UUID / timestamp helpers
// ---------------------------------------------------------------------------

/// Generate a simple unique identifier.
/// Format: "{timestamp}-{random_hex}" (e.g., "1704067200-a3f9c2")
fn generate_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let timestamp = current_timestamp();
    let random: u32 = rng.r#gen();
    format!("{}-{:06x}", timestamp, random & 0xFFFFFF)
}

/// Get current Unix timestamp in seconds.
fn current_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ---------------------------------------------------------------------------
// Unified save file operations
// ---------------------------------------------------------------------------

/// Creates a new empty unified save file.
fn new_unified_save() -> UnifiedSaveFile {
    UnifiedSaveFile {
        metadata: SaveMetadata {
            version: 2,
            last_active_wizard_id: None,
        },
        player: PlayerMetaProgress::default(),
        wizards: Vec::new(),
    }
}

/// Load the unified save file from localStorage.
pub(crate) fn load_unified_save() -> Option<UnifiedSaveFile> {
    let encoded = storage::load_unified_save().ok()?;
    let obfuscated = from_base64(&encoded)?;
    let deobfuscated = deobfuscate(&obfuscated);
    let toml_string = String::from_utf8(deobfuscated).ok()?;

    match toml::from_str::<UnifiedSaveFile>(&toml_string) {
        Ok(data) => Some(data),
        Err(e) => {
            warn!("Failed to parse unified save file: {}", e);
            None
        }
    }
}

/// Save the unified save file to localStorage.
fn save_unified(save_file: &UnifiedSaveFile) {
    match toml::to_string_pretty(save_file) {
        Ok(toml_string) => {
            let obfuscated = obfuscate(toml_string.as_bytes());
            let encoded = to_base64(&obfuscated);
            if let Err(e) = storage::save_unified_save(&encoded) {
                error!("Failed to save unified save file: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to serialize unified save file: {}", e);
        }
    }
}

/// Get the saved wizard for a specific wizard type (if one exists).
pub(crate) fn get_wizard_by_type(wizard_type: WizardType) -> Option<WizardSave> {
    let save_file = load_unified_save()?;
    save_file
        .wizards
        .into_iter()
        .find(|w| w.wizard_type == wizard_type)
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
    active_save.0 = Some(wizard.id.clone());
    true
}

/// Create a new wizard and add it to the save file.
/// Returns the new wizard's ID.
pub(crate) fn create_wizard(wizard_type: WizardType) -> String {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    let wizard = WizardSave {
        id: generate_id(),
        wizard_type,
        current_level: 1,
        highest_level_achieved: 1,
        created_at: current_timestamp(),
        last_played_at: current_timestamp(),
        efficiency_ratios: HashMap::new(),
        action_bar_slots: [None; 5],
    };

    let id = wizard.id.clone();
    save_file.wizards.push(wizard);
    save_unified(&save_file);
    id
}

/// Save the current GameConfig back to the active wizard in the unified save.
pub(crate) fn save_config_to_active_wizard(config: &GameConfig, active_save: &ActiveSave) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.wizard_type = config.wizard_type;
        wizard.current_level = config.current_level;
        wizard.highest_level_achieved = config.highest_level_achieved;
        wizard.efficiency_ratios = config.efficiency_ratios.clone();
        wizard.action_bar_slots = config.action_bar_slots;
        wizard.last_played_at = current_timestamp();
    }

    save_file.metadata.last_active_wizard_id = Some(wizard_id.clone());
    save_unified(&save_file);
}

/// Increment meta-progression counters on victory.
#[allow(dead_code)]
pub(crate) fn increment_levels_completed() {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_levels_completed += 1;
    save_unified(&save_file);
}

/// Increment total games played counter.
#[allow(dead_code)]
pub(crate) fn increment_games_played() {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_games_played += 1;
    save_unified(&save_file);
}

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
