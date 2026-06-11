use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Display mode options for the game window.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// Standard windowed mode with title bar and borders
    #[default]
    Windowed,
    /// Borderless fullscreen (covers the screen without exclusive mode)
    BorderlessFullscreen,
}

/// Window settings for serialization to/from TOML.
///
/// During runtime, Bevy's `Window` component is the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WindowConfig {
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Display mode (windowed or borderless fullscreen)
    #[serde(default)]
    pub display_mode: DisplayMode,
    /// Scale factor override (None uses OS default)
    pub scale_factor: Option<f64>,
    /// Saved window X position in physical pixels (None = let OS decide)
    #[serde(default)]
    pub position_x: Option<i32>,
    /// Saved window Y position in physical pixels (None = let OS decide)
    #[serde(default)]
    pub position_y: Option<i32>,
    /// Saved windowed mode width in logical pixels
    #[serde(default = "default_windowed_width")]
    pub windowed_width: f32,
    /// Saved windowed mode height in logical pixels
    #[serde(default = "default_windowed_height")]
    pub windowed_height: f32,
}

const DEFAULT_WINDOWED_WIDTH: f32 = 1920.0;
const DEFAULT_WINDOWED_HEIGHT: f32 = 1080.0;

fn default_windowed_width() -> f32 {
    DEFAULT_WINDOWED_WIDTH
}
fn default_windowed_height() -> f32 {
    DEFAULT_WINDOWED_HEIGHT
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            vsync: VsyncMode::default(),
            display_mode: DisplayMode::default(),
            scale_factor: Some(1.0),
            position_x: None,
            position_y: None,
            windowed_width: DEFAULT_WINDOWED_WIDTH,
            windowed_height: DEFAULT_WINDOWED_HEIGHT,
        }
    }
}

/// Runtime resource tracking the last windowed-mode size and position.
/// Saved before entering fullscreen/borderless so we can restore when switching back.
#[derive(Resource, Debug, Clone)]
pub(crate) struct SavedWindowedGeometry {
    pub width: f32,
    pub height: f32,
    pub position: Option<IVec2>,
    /// When set, apply this display mode change on the next frame.
    /// Deferred by one frame to avoid scissor rect / render target size mismatch
    /// when the window mode and resolution change in the same frame.
    pub pending_mode_change: Option<DisplayMode>,
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

/// Temporary structure for TOML serialization only.
///
/// This is NOT a runtime resource. It only exists during:
/// 1. Startup: Load from disk → apply to Bevy components
/// 2. Save: Read from Bevy components → serialize to disk
///
/// During runtime, Bevy components are the single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ConfigFile {
    /// Window configuration settings
    pub window: WindowConfig,
    /// Audio configuration settings
    pub audio: AudioConfig,
    /// Game configuration settings (includes all user preferences)
    pub game: super::game_config::GameConfig,
    /// Input key bindings
    #[serde(default)]
    pub controls: crate::config::input_bindings::InputBindings,
}
