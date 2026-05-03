use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::cauldron::brews::Ingredient;
use crate::game::units::UnitType;
use crate::game::units::wizard::components::Spell;

use super::progress::{keyed_hash, load_verified_progress, to_hex};
use super::resources::{
    ActiveSave, GameConfig, WizardType, deserialize_action_bar, serialize_action_bar,
};
use super::storage;

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
    /// Integrity signature (keyed hash over player + wizard data).
    /// Empty for saves created before this feature was added.
    #[serde(default)]
    pub(crate) signature: String,
}

/// Player-level meta-progression (account-wide, not per-wizard).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PlayerMetaProgress {
    pub(crate) total_levels_completed: u32,
    pub(crate) total_games_played: u32,
    #[serde(default)]
    pub(crate) total_defenders_killed: u32,
    #[serde(default)]
    pub(crate) total_attackers_killed: u32,
    #[serde(default)]
    pub(crate) total_undead_killed: u32,
    #[serde(default)]
    pub(crate) unlocked_achievements: Vec<String>,
    #[serde(default)]
    pub(crate) unlocked_content: UnlockedContent,
    /// Spendable Arcane Insight currency (earned from battles).
    #[serde(default)]
    pub(crate) arcane_insight: u32,
    /// Per-spell research progress: spell debug name → insight invested so far.
    #[serde(default)]
    pub(crate) spell_research_progress: HashMap<String, u32>,
    /// Per-spell talent progress: spell debug name → cumulative usage progress.
    #[serde(default)]
    pub(crate) spell_talent_progress: HashMap<String, u32>,
    /// Per-spell talent selections: spell debug name → Vec<i8> of length 3
    /// where -1 = no selection, 0-2 = choice index.
    #[serde(default)]
    pub(crate) spell_talent_selections: HashMap<String, Vec<i8>>,
    /// Tutorial IDs that have been completed.
    #[serde(default)]
    pub(crate) completed_tutorials: Vec<String>,
    /// Toggle modifier IDs that have been permanently unlocked with Insight.
    #[serde(default)]
    pub(crate) unlocked_toggles: Vec<String>,
    /// Permanent insight bonus levels: bonus stat id → level (0-5).
    #[serde(default)]
    pub(crate) insight_bonuses: HashMap<String, u8>,
}

/// Tracks which content the player has unlocked (spells, ingredients, wizard types).
/// Defaults to everything unlocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UnlockedContent {
    #[serde(default = "UnlockedContent::all_spells")]
    pub(crate) spells: Vec<String>,
    #[serde(default = "UnlockedContent::all_ingredients")]
    pub(crate) ingredients: Vec<String>,
    #[serde(default = "UnlockedContent::default_wizard_types")]
    pub(crate) wizard_types: Vec<String>,
    #[serde(default)]
    pub(crate) combos: Vec<String>,
    #[serde(default = "UnlockedContent::default_units")]
    pub(crate) units: Vec<String>,
}

impl UnlockedContent {
    fn all_spells() -> Vec<String> {
        Spell::all().iter().map(|s| format!("{:?}", s)).collect()
    }

    fn default_spells() -> Vec<String> {
        Spell::default_unlocked()
            .iter()
            .map(|s| format!("{:?}", s))
            .collect()
    }

    fn all_ingredients() -> Vec<String> {
        Ingredient::all()
            .iter()
            .map(|i| format!("{:?}", i))
            .collect()
    }

    fn default_ingredients() -> Vec<String> {
        vec![format!("{:?}", Ingredient::Lavender)]
    }

    fn default_units() -> Vec<String> {
        UnitType::all()
            .iter()
            .filter(|u| u.is_default_unlocked())
            .map(|u| format!("{:?}", u))
            .collect()
    }

    fn default_wizard_types() -> Vec<String> {
        WizardType::all()
            .iter()
            .filter(|w| **w == WizardType::BoringOleMage)
            .map(|w| format!("{:?}", w))
            .collect()
    }
}

impl Default for UnlockedContent {
    fn default() -> Self {
        Self {
            spells: Self::default_spells(),
            ingredients: Self::default_ingredients(),
            wizard_types: Self::default_wizard_types(),
            combos: Vec::new(),
            units: Self::default_units(),
        }
    }
}

// ---------------------------------------------------------------------------
// Achievement definitions
// ---------------------------------------------------------------------------

pub(crate) use super::achievement_id::AchievementId;

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

/// Clear all progress and start a fresh save, keeping the previous save as a
/// single rollback backup at `<save>.cleared` (overwriting any prior backup).
pub(crate) fn clear_progress() {
    flush_save_cache();
    if let Err(e) = storage::move_unified_save_to_cleared_backup() {
        warn!("Failed to back up save before clearing progress: {e}");
    }
    save_unified(&new_unified_save());
    flush_save_cache();
}

/// Serializable wall placement data for permanent walls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedWall {
    pub(crate) center_x: f32,
    pub(crate) center_z: f32,
    pub(crate) half_length: f32,
    pub(crate) half_width: f32,
    pub(crate) forward_x: f32,
    pub(crate) forward_z: f32,
    pub(crate) height: f32,
    pub(crate) empowerment: f32,
}

/// Serializable crystal placement data for permanent crystals (Auto-Crystal talent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedCrystal {
    pub(crate) x: f32,
    pub(crate) z: f32,
    pub(crate) range: f32,
    pub(crate) empowerment: f32,
}

/// Sparse trampling grid data for non-zero cells.
///
/// Stored as base64 of a packed byte stream where each non-zero cell is
/// 5 bytes (`u32` little-endian index followed by `u8` intensity). This is
/// ~5.6× smaller in TOML than serializing one line per cell and dramatically
/// cheaper to write during save flushes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct SavedTrampling {
    /// Grid resolution (cells per side). Used to detect grid size changes.
    #[serde(default)]
    pub(crate) grid_size: usize,
    /// Base64-encoded packed cells.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) cells_b64: String,
}

/// Serializable flora placement data for battlefield flowers/plants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedFlora {
    pub(crate) id: u32,
    pub(crate) x: f32,
    pub(crate) z: f32,
    pub(crate) sprite_index: u8,
}

/// Serializable tree placement data for persistent trees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedTree {
    pub(crate) x: f32,
    pub(crate) z: f32,
    /// Size multiplier (1.0 = default, varies ~0.8–1.2).
    #[serde(default = "default_scale")]
    pub(crate) scale: f32,
    /// Sprite variant index (0–4) into the tree sprite sheet.
    #[serde(default)]
    pub(crate) sprite_index: u8,
}

/// Serializable pond placement data for persistent ponds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedPond {
    pub(crate) x: f32,
    pub(crate) z: f32,
    pub(crate) radius: f32,
}

/// Serializable bush placement data for persistent bushes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedBush {
    pub(crate) x: f32,
    pub(crate) z: f32,
    /// Size multiplier (1.0 = default, varies ~0.8–1.2).
    #[serde(default = "default_scale")]
    pub(crate) scale: f32,
    /// Sprite variant index (0–9) into the bush sprite sheet.
    #[serde(default)]
    pub(crate) sprite_index: u8,
}

/// Serializable boulder placement data for terrain-spawned boulders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedBoulder {
    pub(crate) x: f32,
    pub(crate) z: f32,
    /// Size multiplier (1.0 = default, varies ~0.8–1.2).
    #[serde(default = "default_scale")]
    pub(crate) scale: f32,
    /// Sprite variant index (0–9) into the boulder sprite sheet.
    #[serde(default)]
    pub(crate) sprite_index: u8,
}

/// Snapshot of all terrain for a specific level, saved on victory.
/// Used to restore terrain when time traveling in Endless mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct SavedLevelTerrain {
    #[serde(default)]
    pub(crate) trees: Vec<SavedTree>,
    #[serde(default)]
    pub(crate) ponds: Vec<SavedPond>,
    #[serde(default)]
    pub(crate) bushes: Vec<SavedBush>,
    #[serde(default)]
    pub(crate) boulders: Vec<SavedBoulder>,
    #[serde(default)]
    pub(crate) walls: Vec<SavedWall>,
    #[serde(default)]
    pub(crate) crystals: Vec<SavedCrystal>,
    #[serde(default)]
    pub(crate) flora: Vec<SavedFlora>,
}

fn default_scale() -> f32 {
    1.0
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
    #[serde(default)]
    pub(crate) saved_walls: Vec<SavedWall>,
    #[serde(default)]
    pub(crate) saved_crystals: Vec<SavedCrystal>,
    #[serde(default)]
    pub(crate) saved_flora: Vec<SavedFlora>,
    /// Trampling grid state (mud trails on battlefield). Persists across endless levels.
    #[serde(default)]
    pub(crate) saved_trampling: SavedTrampling,
    /// Roguelite run history (last 20 runs). Added in game mode update.
    #[serde(default)]
    pub(crate) roguelite: RogueliteData,
    /// Best stats achieved per endless level. Added in game mode update.
    #[serde(default)]
    pub(crate) endless_best_stats: HashMap<String, EndlessLevelBest>,
    /// Per-level terrain snapshots for Endless time travel.
    /// Key is the level number (as string). Value is the terrain state
    /// at the END of that level (= start of the next level).
    #[serde(default)]
    pub(crate) terrain_per_level: HashMap<String, SavedLevelTerrain>,
}

/// Per-wizard roguelite data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RogueliteData {
    pub(crate) run_history: Vec<RogueliteRun>,
    /// A dormant roguelite run that can be resumed from the wizard tower.
    /// Set when the player is between levels. Cleared when the run ends
    /// (victory at max level, explicit abandon, or exit-to-menu mid-level).
    #[serde(default)]
    pub(crate) current_run: Option<SavedRogueliteRun>,
}

/// A persistable snapshot of an in-progress roguelite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SavedRogueliteRun {
    pub(crate) started_at: u64,
    pub(crate) current_level: u32,
    pub(crate) wizard_type: crate::config::WizardType,
    pub(crate) level_stats: Vec<crate::game::game_mode::components::LevelRunStats>,
    pub(crate) modifiers: Option<crate::game::game_mode::components::RogueliteModifiers>,
    pub(crate) seed: Option<u64>,
    pub(crate) active_toggles: Vec<String>,
    /// True if accessibility assists (game speed != 1.0 or aim assist) were active.
    #[serde(default)]
    pub(crate) accessibility_assists: bool,
}

/// A completed roguelite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RogueliteRun {
    pub(crate) victory: bool,
    pub(crate) levels_completed: u32,
    pub(crate) started_at: u64,
    pub(crate) ended_at: u64,
    #[serde(default)]
    pub(crate) wizard_type: crate::config::WizardType,
    /// Whether this run is permanently saved (exempt from history trimming).
    #[serde(default)]
    pub(crate) saved: bool,
    pub(crate) level_stats: Vec<crate::game::game_mode::components::LevelRunStats>,
    /// Player-chosen modifiers for this run (None for runs before modifiers existed).
    #[serde(default)]
    pub(crate) modifiers: Option<crate::game::game_mode::components::RogueliteModifiers>,
    /// Seed used for this run (deterministic terrain/unit generation).
    #[serde(default)]
    pub(crate) seed: Option<u64>,
    /// Toggle modifier IDs that were active during this run.
    #[serde(default)]
    pub(crate) active_toggles: Vec<String>,
    /// True if accessibility assists (game speed != 1.0 or aim assist) were active.
    #[serde(default)]
    pub(crate) accessibility_assists: bool,
}

/// Best stats achieved on a single endless level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EndlessLevelBest {
    pub(crate) best_efficiency: f32,
    pub(crate) attackers_killed: u32,
    pub(crate) undead_killed: u32,
    pub(crate) defenders_lost: u32,
    pub(crate) elapsed_time: f32,
}

// Encoding helpers moved to save_encoding.rs (Phase 11)
pub(crate) use super::save_encoding::{current_timestamp, from_base64, to_base64};
use super::save_encoding::{deobfuscate, generate_id, obfuscate};

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
        player: PlayerMetaProgress::default(),
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
pub(crate) fn update_endless_best_stats(
    active_save: &ActiveSave,
    stats: &crate::game::game_mode::components::LevelRunStats,
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
                },
            );
        }
    }

    save_unified(&save_file);
}

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
