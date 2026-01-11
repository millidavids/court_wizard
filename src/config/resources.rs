use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Resolution settings for a specific window mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Aspect ratio (e.g., "16:9", "16:10", "4:3", "21:9")
    pub aspect_ratio: String,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            aspect_ratio: "16:9".to_string(),
        }
    }
}

/// Window settings for serialization to/from TOML.
///
/// During runtime, Bevy's `Window` component is the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Resolution for windowed mode
    pub windowed_resolution: Resolution,
    /// Resolution for borderless fullscreen mode
    pub borderless_resolution: Resolution,
    /// Resolution for fullscreen mode
    pub fullscreen_resolution: Resolution,
    /// Display mode (windowed, borderless, or fullscreen)
    pub mode: WindowMode,
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Scale factor override (None uses OS default)
    pub scale_factor: Option<f64>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            windowed_resolution: Resolution::default(),
            borderless_resolution: Resolution {
                width: 1920,
                height: 1080,
                aspect_ratio: "16:9".to_string(),
            },
            fullscreen_resolution: Resolution {
                width: 1920,
                height: 1080,
                aspect_ratio: "16:9".to_string(),
            },
            mode: WindowMode::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_file_serialization() {
        let config = ConfigFile::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");

        // Verify the TOML structure is correct
        assert!(toml_str.contains("[window]"));
        assert!(toml_str.contains("[audio]"));
        assert!(toml_str.contains("[game]"));
        assert!(toml_str.contains("[window.windowed_resolution]"));
    }

    #[test]
    fn test_config_file_deserialization() {
        let toml_str = r#"
            [window]
            mode = "Fullscreen"
            vsync = "Off"
            scale_factor = 2.0

            [window.windowed_resolution]
            width = 1280
            height = 720
            aspect_ratio = "16:9"

            [window.borderless_resolution]
            width = 1920
            height = 1080
            aspect_ratio = "16:9"

            [window.fullscreen_resolution]
            width = 1920
            height = 1080
            aspect_ratio = "16:9"

            [audio]
            master_volume = 0.5
            music_volume = 0.6
            sfx_volume = 0.7

            [game]
            difficulty = "Hard"
        "#;

        let config: ConfigFile = toml::from_str(toml_str).expect("Failed to deserialize");

        // Verify values were parsed correctly
        assert_eq!(config.window.windowed_resolution.width, 1280);
        assert_eq!(config.window.windowed_resolution.aspect_ratio, "16:9");
        assert_eq!(config.window.mode, WindowMode::Fullscreen);
        assert_eq!(config.window.vsync, VsyncMode::Off);
        assert_eq!(config.window.scale_factor, Some(2.0));
        assert_eq!(config.audio.master_volume, 0.5);
        assert_eq!(config.game.difficulty, Difficulty::Hard);
    }

    #[test]
    fn test_config_file_round_trip() {
        let original = ConfigFile::default();
        let toml_str = toml::to_string(&original).expect("Failed to serialize");
        let deserialized: ConfigFile = toml::from_str(&toml_str).expect("Failed to deserialize");

        // Verify serialization and deserialization are symmetrical
        assert_eq!(
            original.window.windowed_resolution.width,
            deserialized.window.windowed_resolution.width
        );
        assert_eq!(
            original.window.windowed_resolution.aspect_ratio,
            deserialized.window.windowed_resolution.aspect_ratio
        );
        assert_eq!(original.window.mode, deserialized.window.mode);
        assert_eq!(original.window.vsync, deserialized.window.vsync);
        assert_eq!(original.game.difficulty, deserialized.game.difficulty);
    }
}
