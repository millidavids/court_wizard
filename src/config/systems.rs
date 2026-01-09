use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection, Window as BevyWindow,
    WindowMode as BevyWindowMode, WindowResized,
};
use std::fs;

use super::helper::calculate_aspect_ratio;
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
            && let Err(e) = fs::write(&config_path.0, toml_string)
        {
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
    // Get the appropriate resolution based on window mode
    let resolution = match config.mode {
        WindowMode::Windowed => &config.windowed_resolution,
        WindowMode::Borderless => &config.borderless_resolution,
        WindowMode::Fullscreen => &config.fullscreen_resolution,
    };

    // Apply resolution
    window
        .resolution
        .set(resolution.width as f32, resolution.height as f32);

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
        resolution.width, resolution.height, window.mode
    );
}

/// System that marks save as pending when window is resized.
///
/// This system runs during the `Update` schedule and listens for
/// `WindowResized` events. Instead of saving immediately, it resets
/// the debounce timer to wait for resize activity to stop.
///
/// # Arguments
///
/// * `resize_events` - Message reader for window resize events
/// * `debounce_timer` - Debounce timer resource
pub fn mark_save_pending_on_resize(
    mut resize_events: MessageReader<WindowResized>,
    mut debounce_timer: ResMut<SaveDebounceTimer>,
) {
    // Only mark as pending if there was actually a resize event
    if resize_events.read().count() == 0 {
        return;
    }

    // Reset the timer and mark save as pending
    debounce_timer.timer.reset();
    debounce_timer.pending = true;
}

/// System that saves configuration to disk when SaveConfigEvent is received.
///
/// This system runs during the `Update` schedule and listens for
/// `SaveConfigEvent` messages. When triggered, it saves the complete
/// configuration (window + game settings) to disk immediately.
///
/// # Arguments
///
/// * `save_events` - Message reader for save config events
/// * `windows` - Query for the primary window
/// * `game_config` - Current game configuration resource
/// * `config_path` - Path to the config file
pub fn save_config_on_event(
    mut save_events: MessageReader<super::resources::SaveConfigEvent>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
    game_config: Res<GameConfig>,
    config_path: Res<ConfigPath>,
) {
    // Only save if there was actually a save message
    if save_events.read().count() == 0 {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    persist_config_file(window, &game_config, &config_path.0);
}

/// System that ticks the debounce timer and saves when it expires.
///
/// This system runs during the `Update` schedule and updates the
/// debounce timer. When the timer finishes and a save is pending,
/// it saves the configuration to disk.
///
/// # Arguments
///
/// * `time` - Time resource for delta time
/// * `debounce_timer` - Debounce timer resource
/// * `windows` - Query for the primary window
/// * `game_config` - Current game configuration resource
/// * `config_path` - Path to the config file
pub fn save_config_on_debounce_timer(
    time: Res<Time>,
    mut debounce_timer: ResMut<SaveDebounceTimer>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
    game_config: Res<GameConfig>,
    config_path: Res<ConfigPath>,
) {
    // Only process if a save is pending
    if !debounce_timer.pending {
        return;
    }

    // Tick the timer
    debounce_timer.timer.tick(time.delta());

    // If timer finished, save the config
    if debounce_timer.timer.is_finished() {
        let Ok(window) = windows.single() else {
            return;
        };

        persist_config_file(window, &game_config, &config_path.0);
        debounce_timer.pending = false;
    }
}

/// Saves the complete configuration file to disk.
///
/// This helper function reads the current state from Bevy's Window component
/// and the GameConfig resource, then serializes everything to TOML and writes
/// it to the config file. It preserves resolutions for other window modes.
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
    // Load existing config to preserve resolutions for other window modes
    let mut existing_config = if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(contents) => toml::from_str::<ConfigFile>(&contents).unwrap_or_default(),
            Err(_) => ConfigFile::default(),
        }
    } else {
        ConfigFile::default()
    };

    // Determine current window mode
    let current_mode = match window.mode {
        BevyWindowMode::Windowed => WindowMode::Windowed,
        BevyWindowMode::BorderlessFullscreen(_) => WindowMode::Borderless,
        BevyWindowMode::Fullscreen(_, _) => WindowMode::Fullscreen,
    };

    // Get current resolution
    let width = window.resolution.physical_width();
    let height = window.resolution.physical_height();
    let current_resolution = Resolution {
        width,
        height,
        aspect_ratio: calculate_aspect_ratio(width, height),
    };

    // Update only the resolution for the current window mode
    match current_mode {
        WindowMode::Windowed => existing_config.window.windowed_resolution = current_resolution,
        WindowMode::Borderless => existing_config.window.borderless_resolution = current_resolution,
        WindowMode::Fullscreen => existing_config.window.fullscreen_resolution = current_resolution,
    }

    // Update window mode and other settings
    existing_config.window.mode = current_mode;
    existing_config.window.vsync = match window.present_mode {
        PresentMode::AutoNoVsync => VsyncMode::Off,
        PresentMode::AutoVsync => VsyncMode::Adaptive,
        _ => VsyncMode::On,
    };
    existing_config.window.scale_factor =
        window.resolution.scale_factor_override().map(|f| f as f64);

    // Update game config
    existing_config.game = game_config.clone();

    let config_file = existing_config;

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
