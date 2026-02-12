use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use crate::game::units::wizard::components::Spell;

/// Temporary structure for TOML serialization only.
///
/// This is NOT a runtime resource. It only exists during:
/// 1. Startup: Load from localStorage → apply to Bevy components
/// 2. Save: Read from Bevy components → serialize to localStorage
///
/// During runtime, Bevy components are the single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ConfigFile {
    /// Window configuration settings
    pub window: WindowConfig,
    /// Audio configuration settings
    pub audio: AudioConfig,
    /// Game configuration settings (includes all user preferences)
    pub game: GameConfig,
}

/// VSync (vertical synchronization) mode options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum VsyncMode {
    /// VSync enabled
    #[default]
    On,
    /// VSync disabled
    Off,
    /// Adaptive VSync (falls back to off if frame rate drops)
    Adaptive,
}

/// Window settings for serialization to/from TOML.
///
/// For WASM builds, window size is controlled by the browser canvas via
/// `fit_canvas_to_parent: true`. Only VSync and scale factor are configurable.
///
/// During runtime, Bevy's `Window` component is the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WindowConfig {
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Scale factor override (None uses OS default)
    pub scale_factor: Option<f64>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            vsync: VsyncMode::default(),
            scale_factor: Some(1.0),
        }
    }
}

/// Audio settings for serialization to/from TOML.
///
/// During runtime, Bevy's audio resources are the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AudioConfig {
    /// Master volume level (0.0 = muted, 1.0 = full volume)
    pub master_volume: f32,
    /// Music track volume level (0.0 = muted, 1.0 = full volume)
    pub music_volume: f32,
    /// Sound effects volume level (0.0 = muted, 1.0 = full volume)
    pub sfx_volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.3,
            sfx_volume: 0.8,
        }
    }
}

/// Game difficulty levels.
///
/// Controls the overall challenge level of the game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Difficulty {
    /// Easy difficulty - relaxed gameplay
    Easy,
    /// Normal difficulty - balanced gameplay (default)
    #[default]
    Normal,
    /// Hard difficulty - challenging gameplay
    Hard,
}

/// Wizard class types available for selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum WizardType {
    /// Rune-based caster - the classic wizard type.
    #[default]
    RuneCaster,
    /// Randomancer - spins a roulette wheel for powerful random spells.
    Randomancer,
    /// Arcanorouter - allocates resources between range, mana, power, and speed.
    Arcanorouter,
}

impl WizardType {
    /// Returns the display name for this wizard type.
    pub const fn display_name(&self) -> &'static str {
        match self {
            WizardType::RuneCaster => "RuneCaster",
            WizardType::Randomancer => "Randomancer",
            WizardType::Arcanorouter => "Arcanorouter",
        }
    }

    /// Returns a short description of this wizard type's playstyle.
    pub const fn description(&self) -> &'static str {
        match self {
            WizardType::RuneCaster => "Master rune sequences to empower your spells.",
            WizardType::Randomancer => "Spin the wheel of fate for powerful random spells.",
            WizardType::Arcanorouter => "Route arcane power between range, mana, power, and speed.",
        }
    }

    /// Returns all available wizard types.
    pub const fn all() -> &'static [WizardType] {
        &[
            WizardType::RuneCaster,
            WizardType::Randomancer,
            WizardType::Arcanorouter,
        ]
    }
}

/// Tracks which wizard is currently active by their unique ID.
/// None means no wizard is loaded (main menu before selection).
#[derive(Resource, Default)]
pub struct ActiveSave(pub Option<String>);

/// Default current level for serde deserialization.
fn default_current_level() -> u32 {
    1
}

/// Default highest level achieved for serde deserialization.
fn default_highest_level() -> u32 {
    1
}

/// Default empty efficiency ratios map for serde deserialization.
fn default_efficiency_ratios() -> HashMap<String, f32> {
    HashMap::new()
}

/// Default action bar slots: None for all 5 slots.
fn default_action_bar_slots() -> [Option<Spell>; 5] {
    [None; 5]
}

/// Serialize action bar slots as a map with string keys, filtering out None values.
pub(crate) fn serialize_action_bar<S>(
    slots: &[Option<Spell>; 5],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(slots.len()))?;
    for (idx, slot) in slots.iter().enumerate() {
        if let Some(spell) = slot {
            map.serialize_entry(&idx.to_string(), spell)?;
        }
    }
    map.end()
}

/// Deserialize action bar slots from a map of string indices to spells.
pub(crate) fn deserialize_action_bar<'de, D>(
    deserializer: D,
) -> Result<[Option<Spell>; 5], D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Spell> = HashMap::deserialize(deserializer).unwrap_or_default();
    let mut slots = [None; 5];
    for (idx_str, spell) in map {
        if let Ok(idx) = idx_str.parse::<usize>()
            && idx < 5
        {
            slots[idx] = Some(spell);
        }
    }
    Ok(slots)
}

/// Game configuration resource - runtime source of truth for all user settings.
///
/// This IS a runtime Bevy resource that holds all user-configurable settings:
/// - VSync mode
/// - Audio volumes (master, music, SFX)
/// - Game difficulty
/// - Global brightness
///
/// Window size/mode is NOT included as it's managed by the browser canvas.
/// Changes to this resource are automatically persisted to localStorage.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use court_wizard::config::{GameConfig, Difficulty};
///
/// fn change_difficulty(mut config: ResMut<GameConfig>) {
///     config.difficulty = Difficulty::Hard;
///     // Automatically persists to localStorage
/// }
/// ```
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameConfig {
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Master volume level (0.0 = muted, 1.0 = full volume)
    pub master_volume: f32,
    /// Music track volume level (0.0 = muted, 1.0 = full volume)
    pub music_volume: f32,
    /// Sound effects volume level (0.0 = muted, 1.0 = full volume)
    pub sfx_volume: f32,
    /// Game difficulty setting
    pub difficulty: Difficulty,
    /// Global brightness multiplier (0.1 = darkest to prevent soft-lock, 1.0 = normal, 2.0 = brightest)
    pub brightness: f32,
    /// Current level - restored on game start after page reload
    #[serde(default = "default_current_level")]
    pub current_level: u32,
    /// Highest level achieved across all playthroughs (high score marker)
    #[serde(default = "default_highest_level")]
    pub highest_level_achieved: u32,
    /// Efficiency ratios per level (defenders lost / total defenders at start)
    /// Key: level number as string, Value: efficiency ratio (0.0 = all defenders lost, 1.0 = no defenders lost)
    #[serde(default = "default_efficiency_ratios")]
    pub efficiency_ratios: HashMap<String, f32>,
    /// Action bar spell slots (0-4 correspond to keys 1-5)
    /// None = empty slot, Some(spell) = assigned spell
    #[serde(
        default = "default_action_bar_slots",
        serialize_with = "serialize_action_bar",
        deserialize_with = "deserialize_action_bar"
    )]
    pub action_bar_slots: [Option<Spell>; 5],
    /// Active wizard type for the current save
    #[serde(default)]
    pub wizard_type: WizardType,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            vsync: VsyncMode::default(),
            master_volume: 1.0,
            music_volume: 0.3,
            sfx_volume: 0.8,
            difficulty: Difficulty::default(),
            brightness: 1.0,
            current_level: 1,
            highest_level_achieved: 1,
            efficiency_ratios: HashMap::new(),
            action_bar_slots: [None; 5],
            wizard_type: WizardType::default(),
        }
    }
}

/// Resource that tracks debounce timer for automatic config saving.
///
/// This prevents excessive file writes during window resizing by waiting
/// for a period of inactivity before saving to disk.
#[derive(Resource)]
pub(crate) struct SaveDebounceTimer {
    /// Timer that counts down after a window resize event
    pub timer: Timer,
    /// Whether a save is pending after the timer expires
    pub pending: bool,
}

impl Default for SaveDebounceTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            pending: false,
        }
    }
}
