use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Root configuration file structure for TOML serialization.
/// This is the complete file format - not a runtime resource.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    pub window: WindowConfig,
    pub audio: AudioConfig,
    pub game: GameConfig,
}

/// Window settings for serialization to/from TOML.
/// During runtime, Bevy's Window component is the source of truth.
/// This struct is only used for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Display mode: "windowed", "borderless", or "fullscreen"
    pub mode: String,
    /// VSync mode: "on", "off", or "adaptive"
    pub vsync: String,
    /// Scale factor override (None uses OS default)
    pub scale_factor: Option<f64>,
}

/// Audio settings for serialization to/from TOML.
/// During runtime, Bevy's audio resources are the source of truth.
/// This struct is only used for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

/// Game difficulty levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

/// Custom game configuration - this IS a runtime resource.
/// Source of truth for game-specific settings like difficulty.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GameConfig {
    /// Game difficulty setting
    pub difficulty: Difficulty,
    // Future: Add more game-specific settings here
}

/// Resource holding the path to the config file
#[derive(Resource)]
pub struct ConfigPath(pub std::path::PathBuf);
