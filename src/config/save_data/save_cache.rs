use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;

use super::super::progress::{keyed_hash, to_hex};
use super::super::save_encoding::{deobfuscate, obfuscate};
use super::super::storage;
use super::save_structs::{SaveMetadata, UnifiedSaveFile, UnlockedContent};

pub(crate) use super::super::save_encoding::{current_timestamp, from_base64, to_base64};

// ---------------------------------------------------------------------------
// In-memory save cache
// ---------------------------------------------------------------------------

/// In-memory cache of the unified save file.
/// Eliminates redundant disk I/O by caching the deserialized save data.
/// All reads come from cache (loading from disk only on first access).
/// All writes update the cache and mark it dirty for deferred flushing.
static SAVE_CACHE: Mutex<Option<UnifiedSaveFile>> = Mutex::new(None);

/// Whether the cache has been modified since the last disk flush.
static SAVE_DIRTY: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Unified save file operations
// ---------------------------------------------------------------------------

/// Creates a new empty unified save file.
pub(crate) fn new_unified_save() -> UnifiedSaveFile {
    UnifiedSaveFile {
        metadata: SaveMetadata {
            version: 2,
            last_active_wizard_id: None,
            signature: String::new(),
        },
        player: Default::default(),
        wizards: Vec::new(),
    }
}

/// Load the unified save file, using the in-memory cache when available.
/// Only reads from disk on the first call or after cache invalidation.
pub(crate) fn load_unified_save() -> Option<UnifiedSaveFile> {
    // Try cache first
    match SAVE_CACHE.lock() {
        Ok(cache) => {
            if let Some(ref cached) = *cache {
                return Some(cached.clone());
            }
        }
        Err(e) => {
            warn!("Save cache mutex poisoned on read, loading from disk: {e}");
        }
    }

    // Cache miss — load from disk
    let data = load_from_disk()?;

    // Populate cache
    match SAVE_CACHE.lock() {
        Ok(mut cache) => *cache = Some(data.clone()),
        Err(e) => warn!("Save cache mutex poisoned on populate: {e}"),
    }

    Some(data)
}

/// Save the unified save file to the in-memory cache.
/// The data will be flushed to disk by the periodic flush system.
pub(crate) fn save_unified(save_file: &UnifiedSaveFile) {
    match SAVE_CACHE.lock() {
        Ok(mut cache) => {
            *cache = Some(save_file.clone());
            SAVE_DIRTY.store(true, Ordering::Release);
        }
        Err(e) => warn!("Save cache mutex poisoned on write: {e}"),
    }
}

/// Flush the in-memory cache to disk if dirty.
/// Called by the periodic flush system and on app exit.
pub(crate) fn flush_save_cache() {
    if !SAVE_DIRTY.load(Ordering::Acquire) {
        return;
    }

    let data = match SAVE_CACHE.lock() {
        Ok(cache) => cache.clone(),
        Err(e) => {
            warn!("Save cache mutex poisoned on flush: {e}");
            return;
        }
    };

    if let Some(save_file) = data {
        write_to_disk(save_file);
        SAVE_DIRTY.store(false, Ordering::Release);
    }
}

/// Check if the save cache has unflushed changes.
pub(crate) fn save_cache_is_dirty() -> bool {
    SAVE_DIRTY.load(Ordering::Acquire)
}

// ---------------------------------------------------------------------------
// Disk I/O (internal — all external access goes through the cache above)
// ---------------------------------------------------------------------------

/// Load the unified save file directly from disk.
fn load_from_disk() -> Option<UnifiedSaveFile> {
    let encoded = storage::load_unified_save().ok()?;
    let obfuscated = from_base64(&encoded)?;
    let deobfuscated = deobfuscate(&obfuscated);
    let toml_string = String::from_utf8(deobfuscated).ok()?;

    match toml::from_str::<UnifiedSaveFile>(&toml_string) {
        Ok(mut data) => {
            // Verify integrity signature (warn but don't block — never lock player out)
            let expected_sig = compute_save_signature(&data);
            if !data.metadata.signature.is_empty() && data.metadata.signature != expected_sig {
                warn!("Save file integrity check failed — data may have been tampered with");
            }

            // Migrate: ensure default spells are always unlocked in existing saves
            for default_spell in UnlockedContent::default_spells() {
                if !data.player.unlocked_content.spells.contains(&default_spell) {
                    data.player.unlocked_content.spells.push(default_spell);
                }
            }
            // Migrate: rename "Battlemage" unlock key to "Swordcerer"
            for t in &mut data.player.unlocked_content.wizard_types {
                if t == "Battlemage" {
                    *t = "Swordcerer".to_string();
                }
            }
            Some(data)
        }
        Err(e) => {
            warn!("Failed to parse unified save file: {}", e);
            None
        }
    }
}

/// Write the unified save file directly to disk with integrity signature.
fn write_to_disk(mut save_file: UnifiedSaveFile) {
    save_file.metadata.signature = compute_save_signature(&save_file);

    match toml::to_string_pretty(&save_file) {
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

/// Compute an integrity signature over the player and wizard data.
/// Uses the existing keyed_hash (SipHash-style) from progress.rs.
fn compute_save_signature(save: &UnifiedSaveFile) -> String {
    let player_toml = toml::to_string(&save.player).unwrap_or_default();
    let wizards_toml = toml::to_string(&save.wizards).unwrap_or_default();
    let combined = format!("{}{}", player_toml, wizards_toml);
    let hash = keyed_hash(combined.as_bytes());
    to_hex(hash)
}
