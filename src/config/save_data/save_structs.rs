use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::cauldron::brews::Ingredient;
use crate::game::units::UnitType;
use crate::game::units::wizard::components::Spell;

use super::super::resources::{WizardType, deserialize_action_bar, serialize_action_bar};

// Re-export AchievementId at this module level so save_data::AchievementId resolves.
pub(crate) use super::super::achievement_id::AchievementId;

/// Unified save file containing all wizards and player meta-progression.
/// Stored as a single file on disk (obfuscated TOML encoded as base64).
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
    pub(crate) fn all_spells() -> Vec<String> {
        Spell::all()
            .iter()
            .map(|s| s.save_key().to_string())
            .collect()
    }

    pub(crate) fn default_spells() -> Vec<String> {
        Spell::default_unlocked()
            .iter()
            .map(|s| s.save_key().to_string())
            .collect()
    }

    pub(crate) fn all_ingredients() -> Vec<String> {
        Ingredient::all()
            .iter()
            .map(|i| format!("{:?}", i))
            .collect()
    }

    fn default_ingredients() -> Vec<String> {
        vec![Ingredient::Lavender.save_key().to_string()]
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

pub(crate) fn default_scale() -> f32 {
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
    /// True if a co-op partner was connected during this run.
    #[serde(default)]
    pub(crate) played_coop: bool,
    /// Co-op partner's Steam display name (if Steam + co-op), else None.
    #[serde(default)]
    pub(crate) coop_peer_name: Option<String>,
}

/// Best stats achieved on a single endless level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EndlessLevelBest {
    pub(crate) best_efficiency: f32,
    pub(crate) attackers_killed: u32,
    pub(crate) undead_killed: u32,
    pub(crate) defenders_lost: u32,
    pub(crate) elapsed_time: f32,
    /// True if a co-op partner was connected when this best was set.
    #[serde(default)]
    pub(crate) played_coop: bool,
    /// Co-op partner's Steam display name (if Steam + co-op), else None.
    #[serde(default)]
    pub(crate) coop_peer_name: Option<String>,
}
