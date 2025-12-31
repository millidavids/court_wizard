use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection, Window, WindowMode,
    WindowResized,
};
use std::fs;

use super::resources::*;

/// System that loads configuration from TOML file at startup and applies settings.
/// - Applies WindowConfig to Bevy's Window component
/// - Inserts GameConfig as a resource
pub fn load_and_apply_config(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
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
        if let Ok(toml_string) = toml::to_string_pretty(&config) {
            let _ = fs::write(&config_path.0, toml_string);
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

/// Helper function to apply WindowConfig to a Bevy Window
fn apply_window_config(config: &WindowConfig, window: &mut Window) {
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
    window.mode = match config.mode.as_str() {
        "fullscreen" => {
            WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current)
        }
        "borderless" => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        _ => WindowMode::Windowed,
    };

    // Apply VSync
    window.present_mode = match config.vsync.as_str() {
        "off" => PresentMode::AutoNoVsync,
        "adaptive" => PresentMode::AutoVsync,
        _ => PresentMode::AutoVsync,
    };

    info!(
        "Applied window config: {}x{} {:?}",
        config.width, config.height, window.mode
    );
}

/// System that persists Window state to disk when window is resized
pub fn persist_window_on_resize(
    mut resize_events: MessageReader<WindowResized>,
    windows: Query<&Window, With<PrimaryWindow>>,
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

/// System that persists GameConfig to disk when it changes
pub fn persist_game_config_on_change(
    game_config: Res<GameConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    config_path: Res<ConfigPath>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    persist_config_file(window, &game_config, &config_path.0);
}

/// Helper function to save complete config file to disk
/// Reads from Bevy's Window and GameConfig resource
fn persist_config_file(window: &Window, game_config: &GameConfig, config_path: &std::path::Path) {
    let window_config = WindowConfig {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        mode: match window.mode {
            WindowMode::Windowed => "windowed".to_string(),
            WindowMode::BorderlessFullscreen(_) => "borderless".to_string(),
            WindowMode::Fullscreen(_, _) => "fullscreen".to_string(),
        },
        vsync: match window.present_mode {
            PresentMode::AutoNoVsync => "off".to_string(),
            PresentMode::AutoVsync => "adaptive".to_string(),
            _ => "on".to_string(),
        },
        scale_factor: window.resolution.scale_factor_override().map(|f| f as f64),
    };

    let config_file = ConfigFile {
        window: window_config,
        audio: AudioConfig::default(), // TODO: Read from Bevy's audio resources
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
