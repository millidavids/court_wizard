use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Temporary structure for TOML serialization only.
///
/// This is NOT a runtime resource. It only exists during:
/// 1. Startup: Load from localStorage → apply to Bevy components
/// 2. Save: Read from Bevy components → serialize to localStorage
///
/// During runtime, Bevy components are the single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
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
pub struct WindowConfig {
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
pub struct AudioConfig {
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
            music_volume: 0.8,
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
/// use the_game::config::{GameConfig, Difficulty};
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
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            vsync: VsyncMode::default(),
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.8,
            difficulty: Difficulty::default(),
            brightness: 1.0,
            current_level: 1,
            highest_level_achieved: 1,
            efficiency_ratios: HashMap::new(),
        }
    }
}

/// Message that triggers saving the current configuration to localStorage.
///
/// Send this message when you want to manually persist the current
/// config state immediately, bypassing the debounce timer.
#[derive(Message)]
pub struct SaveConfigEvent;

/// Message that triggers debounced config save.
///
/// Send this message whenever any configuration changes that should be
/// persisted to localStorage. The ConfigPlugin will debounce these messages
/// and save after 0.5s of inactivity.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use the_game::config::{ConfigChanged, GameConfig, Difficulty};
///
/// fn change_difficulty(
///     mut config: ResMut<GameConfig>,
///     mut events: MessageWriter<ConfigChanged>,
/// ) {
///     config.difficulty = Difficulty::Hard;
///     events.write(ConfigChanged);  // Trigger debounced save
/// }
/// ```
#[derive(Message)]
pub struct ConfigChanged;

/// Resource that tracks debounce timer for automatic config saving.
///
/// This prevents excessive file writes during window resizing by waiting
/// for a period of inactivity before saving to disk.
#[derive(Resource)]
pub struct SaveDebounceTimer {
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
