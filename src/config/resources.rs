use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Root configuration file structure for TOML serialization.
///
/// This is the complete file format that gets serialized to/from `config.toml`.
/// It is not a runtime resource - it's only used as a data transfer object
/// between the disk and Bevy's runtime components/resources.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    /// Window configuration settings
    pub window: WindowConfig,
    /// Audio configuration settings
    pub audio: AudioConfig,
    /// Custom game configuration settings
    pub game: GameConfig,
}

/// Window display mode options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum WindowMode {
    /// Window takes a portion of the screen
    #[default]
    Windowed,
    /// Borderless fullscreen window
    Borderless,
    /// Exclusive fullscreen mode
    Fullscreen,
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
/// During runtime, Bevy's `Window` component is the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Display mode (windowed, borderless, or fullscreen)
    pub mode: WindowMode,
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Scale factor override (None uses OS default)
    pub scale_factor: Option<f64>,
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

/// Custom game configuration resource.
///
/// This IS a runtime Bevy resource and serves as the source of truth
/// for game-specific settings like difficulty. During runtime, systems
/// can query and modify this resource. Changes are automatically
/// persisted to the config file.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use the_game::config::{GameConfig, Difficulty};
///
/// fn change_difficulty(mut config: ResMut<GameConfig>) {
///     config.difficulty = Difficulty::Hard;
///     // Automatically persists to config.toml
/// }
/// ```
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GameConfig {
    /// Game difficulty setting
    pub difficulty: Difficulty,
    // Future: Add more game-specific settings here
}

/// Resource holding the path to the config file.
///
/// This resource is inserted by the `ConfigPlugin` and stores the
/// absolute or relative path to the configuration file on disk.
#[derive(Resource)]
pub struct ConfigPath(pub std::path::PathBuf);
