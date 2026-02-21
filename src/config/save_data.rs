use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::cauldron::brews::Ingredient;
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
}

impl UnlockedContent {
    fn all_spells() -> Vec<String> {
        Spell::all().iter().map(|s| format!("{:?}", s)).collect()
    }

    fn default_spells() -> Vec<String> {
        vec![
            format!("{:?}", Spell::MagicMissile),
            format!("{:?}", Spell::Telekinesis),
        ]
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
        }
    }
}

// ---------------------------------------------------------------------------
// Achievement definitions
// ---------------------------------------------------------------------------

/// Type-safe achievement identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AchievementId {
    FirstVictory,
    FriendlyFire,
    ChainReaction,
    // Defeat & Failure
    TacticalRetreat,
    TheKingIsDead,
    TotalWipe,
    SpeedrunWrongDirection,
    PyrrhicDefeat,
    ItWasGoingSoWell,
    FriendlyFireDepartment,
    AccidentalRegicide,
    // Victory & Progression
    ApprenticeWizard,
    CourtWizard,
    Archmage,
    LegendsSpeakYourName,
    Immortalized,
    TheGrindNeverStops,
    OneMoreLevel,
    IntoTheDeep,
    Absurdity,
    Level100,
    Stubborn,
    ExtremelyStubborn,
    // Meta / Unlocks
    SliderFiddler,
    RandomMagicSurge,
    Qwer,
    // Spell Unlocks
    OutOfRange,
    ScorchedEarth,
    ProtectiveInstincts,
    FriendlyThorns,
}

impl AchievementId {
    /// Returns all achievement variants.
    pub(crate) fn all() -> &'static [AchievementId] {
        &[
            AchievementId::FirstVictory,
            AchievementId::FriendlyFire,
            AchievementId::ChainReaction,
            AchievementId::TacticalRetreat,
            AchievementId::TheKingIsDead,
            AchievementId::TotalWipe,
            AchievementId::SpeedrunWrongDirection,
            AchievementId::PyrrhicDefeat,
            AchievementId::ItWasGoingSoWell,
            AchievementId::FriendlyFireDepartment,
            AchievementId::AccidentalRegicide,
            AchievementId::ApprenticeWizard,
            AchievementId::CourtWizard,
            AchievementId::Archmage,
            AchievementId::LegendsSpeakYourName,
            AchievementId::Immortalized,
            AchievementId::TheGrindNeverStops,
            AchievementId::OneMoreLevel,
            AchievementId::IntoTheDeep,
            AchievementId::Absurdity,
            AchievementId::Level100,
            AchievementId::Stubborn,
            AchievementId::ExtremelyStubborn,
            AchievementId::SliderFiddler,
            AchievementId::RandomMagicSurge,
            AchievementId::Qwer,
            AchievementId::OutOfRange,
            AchievementId::ScorchedEarth,
            AchievementId::ProtectiveInstincts,
            AchievementId::FriendlyThorns,
        ]
    }

    /// String identifier used for persistence.
    pub(crate) fn id(&self) -> &'static str {
        match self {
            AchievementId::FirstVictory => "first_victory",
            AchievementId::FriendlyFire => "friendly_fire",
            AchievementId::ChainReaction => "chain_reaction",
            AchievementId::TacticalRetreat => "tactical_retreat",
            AchievementId::TheKingIsDead => "the_king_is_dead",
            AchievementId::TotalWipe => "total_wipe",
            AchievementId::SpeedrunWrongDirection => "speedrun_wrong_direction",
            AchievementId::PyrrhicDefeat => "pyrrhic_defeat",
            AchievementId::ItWasGoingSoWell => "it_was_going_so_well",
            AchievementId::FriendlyFireDepartment => "friendly_fire_department",
            AchievementId::AccidentalRegicide => "accidental_regicide",
            AchievementId::ApprenticeWizard => "apprentice_wizard",
            AchievementId::CourtWizard => "court_wizard",
            AchievementId::Archmage => "archmage",
            AchievementId::LegendsSpeakYourName => "legends_speak_your_name",
            AchievementId::Immortalized => "immortalized",
            AchievementId::TheGrindNeverStops => "the_grind_never_stops",
            AchievementId::OneMoreLevel => "one_more_level",
            AchievementId::IntoTheDeep => "into_the_deep",
            AchievementId::Absurdity => "absurdity",
            AchievementId::Level100 => "level_100",
            AchievementId::Stubborn => "stubborn",
            AchievementId::ExtremelyStubborn => "extremely_stubborn",
            AchievementId::SliderFiddler => "slider_fiddler",
            AchievementId::RandomMagicSurge => "random_magic_surge",
            AchievementId::Qwer => "qwer",
            AchievementId::OutOfRange => "out_of_range",
            AchievementId::ScorchedEarth => "scorched_earth",
            AchievementId::ProtectiveInstincts => "protective_instincts",
            AchievementId::FriendlyThorns => "friendly_thorns",
        }
    }

    /// Display name shown in the achievement popup.
    pub(crate) fn display_name(&self) -> &'static str {
        match self {
            AchievementId::FirstVictory => "First Victory",
            AchievementId::FriendlyFire => "Friendly Fire",
            AchievementId::ChainReaction => "Chain Reaction",
            AchievementId::TacticalRetreat => "Tactical Retreat",
            AchievementId::TheKingIsDead => "The King is Dead",
            AchievementId::TotalWipe => "Total Wipe",
            AchievementId::SpeedrunWrongDirection => "Speedrun (Wrong Direction)",
            AchievementId::PyrrhicDefeat => "Pyrrhic Defeat",
            AchievementId::ItWasGoingSoWell => "It Was Going So Well",
            AchievementId::FriendlyFireDepartment => "Friendly Fire Department",
            AchievementId::AccidentalRegicide => "Accidental Regicide",
            AchievementId::ApprenticeWizard => "Apprentice Wizard",
            AchievementId::CourtWizard => "Court Wizard",
            AchievementId::Archmage => "Archmage",
            AchievementId::LegendsSpeakYourName => "Legends Speak Your Name",
            AchievementId::Immortalized => "Immortalized",
            AchievementId::TheGrindNeverStops => "The Grind Never Stops",
            AchievementId::OneMoreLevel => "One More Level",
            AchievementId::IntoTheDeep => "Into the Deep",
            AchievementId::Absurdity => "Absurdity",
            AchievementId::Level100 => "Level 100",
            AchievementId::Stubborn => "Stubborn",
            AchievementId::ExtremelyStubborn => "Extremely Stubborn",
            AchievementId::SliderFiddler => "Slider Fiddler",
            AchievementId::RandomMagicSurge => "Random Magic Surge",
            AchievementId::Qwer => "QWER",
            AchievementId::OutOfRange => "Out of Range",
            AchievementId::ScorchedEarth => "Scorched Earth",
            AchievementId::ProtectiveInstincts => "Protective Instincts",
            AchievementId::FriendlyThorns => "Friendly Thorns",
        }
    }

    /// Returns the unlock reward text for this achievement.
    pub(crate) fn unlock_reward(&self) -> Option<&'static str> {
        match self {
            AchievementId::FirstVictory => Some("Grants: 25 Arcane Insight"),
            AchievementId::FriendlyFire => Some("Grants: 15 Arcane Insight"),
            AchievementId::ChainReaction => Some("Grants: 20 Arcane Insight"),
            AchievementId::TheKingIsDead => Some("Grants: 15 Arcane Insight"),
            AchievementId::ApprenticeWizard => Some("Grants: 40 Arcane Insight"),
            AchievementId::CourtWizard => Some("Grants: 60 Arcane Insight"),
            AchievementId::Archmage => Some("Grants: 80 Arcane Insight"),
            AchievementId::LegendsSpeakYourName => Some("Grants: 100 Arcane Insight"),
            AchievementId::SliderFiddler => Some("Unlocks: Arcanorouter wizard"),
            AchievementId::RandomMagicSurge => Some("Unlocks: Randomancer wizard"),
            AchievementId::Qwer => Some("Unlocks: Rune Caster wizard"),
            AchievementId::OutOfRange => Some("Grants: 15 Arcane Insight"),
            AchievementId::ScorchedEarth => Some("Grants: 15 Arcane Insight"),
            AchievementId::ProtectiveInstincts => Some("Grants: 15 Arcane Insight"),
            AchievementId::FriendlyThorns => Some("Grants: 15 Arcane Insight"),
            _ => None,
        }
    }

    /// Returns the Arcane Insight reward for this achievement (0 if none).
    pub(crate) fn insight_reward(&self) -> u32 {
        match self {
            AchievementId::FirstVictory => 25,
            AchievementId::FriendlyFire => 15,
            AchievementId::ChainReaction => 20,
            AchievementId::TheKingIsDead => 15,
            AchievementId::ApprenticeWizard => 40,
            AchievementId::CourtWizard => 60,
            AchievementId::Archmage => 80,
            AchievementId::LegendsSpeakYourName => 100,
            AchievementId::OutOfRange => 15,
            AchievementId::ScorchedEarth => 15,
            AchievementId::ProtectiveInstincts => 15,
            AchievementId::FriendlyThorns => 15,
            _ => 0,
        }
    }

    /// Description shown in the achievement popup.
    pub(crate) fn description(&self) -> &'static str {
        match self {
            AchievementId::FirstVictory => "You won your first battle!",
            AchievementId::FriendlyFire => "You killed a defender with a spell. Oops!",
            AchievementId::ChainReaction => "Killed multiple enemies in quick succession.",
            AchievementId::TacticalRetreat => "A strategic withdrawal to reconsider your options.",
            AchievementId::TheKingIsDead => "You had one job.",
            AchievementId::TotalWipe => {
                "Not a single soldier left standing. Impressive, in a horrible way."
            }
            AchievementId::SpeedrunWrongDirection => "The battle barely started. What happened?",
            AchievementId::PyrrhicDefeat => "Close, but no cigar.",
            AchievementId::ItWasGoingSoWell => "Everything was fine. And then it wasn't.",
            AchievementId::FriendlyFireDepartment => "Maybe stop casting into your own army?",
            AchievementId::AccidentalRegicide => {
                "The defense has been called off on account of you."
            }
            AchievementId::ApprenticeWizard => "You're getting the hang of this.",
            AchievementId::CourtWizard => "The king trusts you. Probably a mistake.",
            AchievementId::Archmage => "Your beard has grown three inches since you started.",
            AchievementId::LegendsSpeakYourName => {
                "Bards write songs. Most of them are inaccurate."
            }
            AchievementId::Immortalized => {
                "Statues of you line the courtyard. They all look wrong."
            }
            AchievementId::TheGrindNeverStops => "You could have learned a real trade by now.",
            AchievementId::OneMoreLevel => "Just one more, you told yourself 9 levels ago.",
            AchievementId::IntoTheDeep => "The attackers are starting to look... different.",
            AchievementId::Absurdity => "This many enemies shouldn't fit on one battlefield.",
            AchievementId::Level100 => {
                "You've been doing this longer than some civilizations lasted."
            }
            AchievementId::Stubborn => {
                "Insanity is doing the same thing over and over. But maybe this time..."
            }
            AchievementId::ExtremelyStubborn => "At this point, the enemies feel bad for you.",
            AchievementId::SliderFiddler => "You adjusted a slider. The Arcanorouter approves.",
            AchievementId::RandomMagicSurge => "Lol, you're so random.",
            AchievementId::Qwer => "Spell-keyboard?",
            AchievementId::OutOfRange => "A defender wandered beyond your reach.",
            AchievementId::ScorchedEarth => "Your own fire claimed a life.",
            AchievementId::ProtectiveInstincts => "You shielded the enemy. On purpose?",
            AchievementId::FriendlyThorns => "Your vines don't discriminate.",
        }
    }
}

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

/// Clear all achievements, lifetime stats, wizard progress, and reset unlocked content to defaults.
pub(crate) fn clear_progress() {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    save_file.player.unlocked_achievements.clear();
    save_file.player.unlocked_content = UnlockedContent::default();
    save_file.player.total_levels_completed = 0;
    save_file.player.total_games_played = 0;
    save_file.player.total_defenders_killed = 0;
    save_file.player.total_attackers_killed = 0;
    save_file.player.total_undead_killed = 0;
    save_file.player.arcane_insight = 0;
    save_file.player.spell_research_progress.clear();

    // Reset all wizard saves to level 1
    for wizard in &mut save_file.wizards {
        wizard.current_level = 1;
        wizard.highest_level_achieved = 1;
        wizard.efficiency_ratios.clear();
        wizard.action_bar_slots = [None; 5];
    }

    save_unified(&save_file);
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
        Ok(mut data) => {
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

    // Validate that all action bar slots contain unlocked spells
    validate_action_bar_slots(&mut config.action_bar_slots);

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

// ---------------------------------------------------------------------------
// LAN IP persistence (plain string in localStorage)
// ---------------------------------------------------------------------------

/// Loads the saved LAN IP address from localStorage.
pub(crate) fn load_lan_ip() -> Option<String> {
    storage::load_lan_ip().ok()
}

/// Saves the LAN IP address to localStorage for future sessions.
pub(crate) fn save_lan_ip(ip: &str) {
    let _ = storage::save_lan_ip(ip);
}
