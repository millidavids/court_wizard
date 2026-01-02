use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection, Window as BevyWindow,
    WindowMode as BevyWindowMode, WindowResized,
};
use std::fs;

use super::resources::*;

/// System that loads configuration from TOML file at startup and applies settings.
///
/// This system runs during the `Startup` schedule and performs the following:
/// 1. Loads the configuration file from disk (or uses defaults if missing/invalid)
/// 2. Applies window settings to Bevy's `Window` component
/// 3. Inserts `GameConfig` as a Bevy resource for runtime access
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for inserting resources
/// * `windows` - Query for the primary window
/// * `config_path` - Resource containing the path to the config file
///
/// # Error Handling
///
/// This system is designed to never fail. If the config file cannot be read
/// or parsed, it falls back to sensible defaults and logs a warning.
pub fn load_and_apply_config(
    mut commands: Commands,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
    config_path: Res<ConfigPath>,
) {
    let config_file = if config_path.0.exists() {
        match fs::read_to_string(&config_path.0) {
            Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                Ok(config) => {
                    info!("Loaded config from {:?}", config_path.0);
                    config
                }
                Err(e) => {
                    warn!("Failed to parse config: {}, using defaults", e);
                    ConfigFile::default()
                }
            },
            Err(e) => {
                warn!("Failed to read config file: {}, using defaults", e);
                ConfigFile::default()
            }
        }
    } else {
        info!("Config file not found, creating with defaults");
        let config = ConfigFile::default();
        // Save defaults to file
        if let Ok(toml_string) = toml::to_string_pretty(&config)
            && let Err(e) = fs::write(&config_path.0, toml_string) {
                warn!("Failed to write default config: {}", e);
            }
        config
    };

    // Apply window config to Bevy's Window
    let Ok(mut window) = windows.single_mut() else {
        warn!("Could not find primary window to apply config");
        return;
    };
    apply_window_config(&config_file.window, &mut window);

    // Insert GameConfig as a resource (our source of truth for game settings)
    commands.insert_resource(config_file.game);
}

/// Applies window configuration settings to Bevy's Window component.
///
/// # Arguments
///
/// * `config` - Window configuration from the config file
/// * `window` - Mutable reference to Bevy's Window component
fn apply_window_config(config: &WindowConfig, window: &mut BevyWindow) {
    // Apply resolution
    window
        .resolution
        .set(config.width as f32, config.height as f32);

    // Apply scale factor override
    if let Some(scale) = config.scale_factor {
        window
            .resolution
            .set_scale_factor_override(Some(scale as f32));
    }

    // Apply window mode
    window.mode = match config.mode {
        WindowMode::Fullscreen => {
            BevyWindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current)
        }
        WindowMode::Borderless => BevyWindowMode::BorderlessFullscreen(MonitorSelection::Current),
        WindowMode::Windowed => BevyWindowMode::Windowed,
    };

    // Apply VSync
    window.present_mode = match config.vsync {
        VsyncMode::Off => PresentMode::AutoNoVsync,
        VsyncMode::Adaptive => PresentMode::AutoVsync,
        VsyncMode::On => PresentMode::AutoVsync,
    };

    info!(
        "Applied window config: {}x{} {:?}",
        config.width, config.height, window.mode
    );
}

/// System that persists Window state to disk when window is resized.
///
/// This system runs during the `Update` schedule and listens for
/// `WindowResized` events. When the window is resized, it saves the
/// complete configuration (window + game settings) to disk.
///
/// # Arguments
///
/// * `resize_events` - Event reader for window resize events
/// * `windows` - Query for the primary window
/// * `game_config` - Current game configuration resource
/// * `config_path` - Path to the config file
pub fn persist_window_on_resize(
    mut resize_events: MessageReader<WindowResized>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
    game_config: Res<GameConfig>,
    config_path: Res<ConfigPath>,
) {
    // Only persist if there was actually a resize event
    if resize_events.read().count() == 0 {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    persist_config_file(window, &game_config, &config_path.0);
}

/// System that persists GameConfig to disk when it changes.
///
/// This system runs during the `Update` schedule when the `GameConfig`
/// resource has been modified. It reads the current window state and
/// saves the complete configuration to disk.
///
/// # Arguments
///
/// * `game_config` - Current game configuration resource
/// * `windows` - Query for the primary window
/// * `config_path` - Path to the config file
pub fn persist_game_config_on_change(
    game_config: Res<GameConfig>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
    config_path: Res<ConfigPath>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    persist_config_file(window, &game_config, &config_path.0);
}

/// Saves the complete configuration file to disk.
///
/// This helper function reads the current state from Bevy's Window component
/// and the GameConfig resource, then serializes everything to TOML and writes
/// it to the config file.
///
/// # Arguments
///
/// * `window` - Reference to Bevy's Window component
/// * `game_config` - Reference to the GameConfig resource
/// * `config_path` - Path where the config file should be saved
///
/// # Error Handling
///
/// Errors are logged but do not cause the function to panic. The game
/// continues running even if config persistence fails.
fn persist_config_file(
    window: &BevyWindow,
    game_config: &GameConfig,
    config_path: &std::path::Path,
) {
    let window_config = WindowConfig {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        mode: match window.mode {
            BevyWindowMode::Windowed => WindowMode::Windowed,
            BevyWindowMode::BorderlessFullscreen(_) => WindowMode::Borderless,
            BevyWindowMode::Fullscreen(_, _) => WindowMode::Fullscreen,
        },
        vsync: match window.present_mode {
            PresentMode::AutoNoVsync => VsyncMode::Off,
            PresentMode::AutoVsync => VsyncMode::Adaptive,
            _ => VsyncMode::On,
        },
        scale_factor: window.resolution.scale_factor_override().map(|f| f as f64),
    };

    let config_file = ConfigFile {
        window: window_config,
        audio: AudioConfig::default(), // TODO: Read from Bevy's audio resources when audio is implemented
        game: game_config.clone(),
    };

    match toml::to_string_pretty(&config_file) {
        Ok(toml_string) => match fs::write(config_path, &toml_string) {
            Ok(_) => {
                info!("Config saved to {:?}", config_path);
            }
            Err(e) => {
                error!("Failed to save config: {}", e);
            }
        },
        Err(e) => {
            error!("Failed to serialize config: {}", e);
        }
    }
}
