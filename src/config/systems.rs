use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection, Window as BevyWindow,
    WindowMode as BevyWindowMode, WindowResized,
};

use super::helper::calculate_aspect_ratio;
use super::resources::*;
use super::storage;

/// System that loads configuration from localStorage at startup and applies settings.
///
/// This system runs during the `Startup` schedule and performs the following:
/// 1. Loads the configuration from browser localStorage (or uses defaults if missing/invalid)
/// 2. Applies window settings to Bevy's `Window` component
/// 3. Inserts `GameConfig` as a Bevy resource for runtime access
///
/// After this system runs, ConfigFile is discarded. Bevy components are the single
/// source of truth during runtime.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for inserting resources
/// * `windows` - Query for the primary window
///
/// # Error Handling
///
/// This system is designed to never fail. If the config cannot be loaded
/// or parsed, it falls back to sensible defaults and logs a warning.
pub fn load_and_apply_config(
    mut commands: Commands,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
) {
    let config_file = match storage::load_config() {
        Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
            Ok(config) => {
                info!("Loaded config from localStorage");
                config
            }
            Err(e) => {
                warn!("Failed to parse config: {}, using defaults", e);
                ConfigFile::default()
            }
        },
        Err(_) => {
            info!("No config in localStorage, using defaults");
            let config = ConfigFile::default();
            // Save defaults to localStorage
            if let Ok(toml_string) = toml::to_string_pretty(&config) {
                let _ = storage::save_config(&toml_string);
            }
            config
        }
    };

    // Apply window config to Bevy's Window
    let Ok(mut window) = windows.single_mut() else {
        warn!("Could not find primary window to apply config");
        return;
    };
    apply_window_config(&config_file.window, &mut window);

    // Insert GameConfig resource (single source of truth for game settings)
    commands.insert_resource(config_file.game);

    // ConfigFile is now discarded - Bevy components are the source of truth
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

/// Bridges WindowResized events to ConfigChanged messages.
///
/// This system translates Bevy's WindowResized event into our unified
/// ConfigChanged message, triggering the debounce timer.
///
/// # Arguments
///
/// * `resize_events` - Message reader for window resize events
/// * `config_changed` - Message writer for config changed messages
pub fn bridge_window_resize_to_config_changed(
    mut resize_events: MessageReader<WindowResized>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if resize_events.read().count() == 0 {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Bridges GameConfig changes to ConfigChanged messages.
///
/// This system detects changes to the GameConfig resource and sends
/// a ConfigChanged message to trigger the debounce timer.
///
/// # Arguments
///
/// * `game_config` - Game configuration resource
/// * `config_changed` - Message writer for config changed messages
pub fn bridge_game_config_to_config_changed(
    game_config: Res<GameConfig>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if !game_config.is_changed() {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Unified debounce trigger for ALL config changes.
///
/// This system listens for the ConfigChanged message and resets the
/// debounce timer. This provides a single unified debouncing mechanism
/// for all types of config changes (window, game, audio, controls, etc.).
///
/// Any system can trigger a debounced save by sending ConfigChanged.
///
/// # Arguments
///
/// * `config_events` - Message reader for config changed messages
/// * `debounce_timer` - Debounce timer resource
pub fn mark_save_on_config_changed(
    mut config_events: MessageReader<ConfigChanged>,
    mut debounce_timer: ResMut<SaveDebounceTimer>,
) {
    if config_events.read().count() == 0 {
        return;
    }

    // Reset timer and mark save pending
    debounce_timer.timer.reset();
    debounce_timer.pending = true;
}

/// Ticks debounce timer and saves to localStorage when expired.
///
/// This system runs every frame during the `Update` schedule. When the
/// debounce timer expires (0.5s of no config changes), it reads the current
/// state from Bevy components and saves to localStorage.
///
/// # Arguments
///
/// * `time` - Time resource for delta time
/// * `debounce_timer` - Debounce timer resource
/// * `windows` - Query for the primary window
/// * `game_config` - Game configuration resource
pub fn save_config_on_debounce_timer(
    time: Res<Time>,
    mut debounce_timer: ResMut<SaveDebounceTimer>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
    game_config: Res<GameConfig>,
) {
    if !debounce_timer.pending {
        return;
    }

    debounce_timer.timer.tick(time.delta());

    if debounce_timer.timer.is_finished() {
        persist_config(windows, game_config);
        debounce_timer.pending = false;
    }
}

/// Manual save trigger (bypasses debounce).
///
/// This system listens for SaveConfigEvent messages and immediately
/// saves the current state to localStorage, bypassing the debounce timer.
/// Useful for critical saves (e.g., on app quit).
///
/// # Arguments
///
/// * `save_events` - Message reader for save config events
/// * `windows` - Query for the primary window
/// * `game_config` - Game configuration resource
pub fn save_config_on_event(
    mut save_events: MessageReader<SaveConfigEvent>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
    game_config: Res<GameConfig>,
) {
    if save_events.read().count() == 0 {
        return;
    }

    persist_config(windows, game_config);
}

/// Saves current state to localStorage by reading from Bevy components.
///
/// This function reads the current state from:
/// - Bevy's Window component (window settings)
/// - GameConfig resource (game settings)
///
/// Then builds a temporary ConfigFile, serializes to TOML, and saves to localStorage.
///
/// # Arguments
///
/// * `windows` - Query for the primary window
/// * `game_config` - Game configuration resource
fn persist_config(windows: Query<&BevyWindow, With<PrimaryWindow>>, game_config: Res<GameConfig>) {
    let Ok(window) = windows.single() else {
        error!("Could not find primary window to save config");
        return;
    };

    // Build ConfigFile from current Bevy component state
    let config_file = build_config_from_components(window, &game_config);

    // Serialize and save
    match toml::to_string_pretty(&config_file) {
        Ok(toml_string) => match storage::save_config(&toml_string) {
            Ok(_) => {
                info!("Config saved to localStorage");
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

/// Builds ConfigFile from current Bevy component state.
///
/// This function reads the current state from Bevy components and constructs
/// a temporary ConfigFile for serialization. The ConfigFile is immediately
/// discarded after serialization - it's not kept in memory.
///
/// # Arguments
///
/// * `window` - Reference to Bevy's Window component
/// * `game_config` - Reference to the GameConfig resource
///
/// # Returns
///
/// A ConfigFile struct populated with current Bevy component state
fn build_config_from_components(window: &BevyWindow, game_config: &GameConfig) -> ConfigFile {
    // Determine current window mode
    let mode = match window.mode {
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

    // Determine VSync mode
    let vsync = match window.present_mode {
        PresentMode::AutoNoVsync => VsyncMode::Off,
        PresentMode::AutoVsync => VsyncMode::Adaptive,
        _ => VsyncMode::On,
    };

    let scale_factor = window.resolution.scale_factor_override().map(|f| f as f64);

    // Build window config, preserving resolutions for other window modes
    let window_config =
        build_window_config_preserving_other_modes(mode, current_resolution, vsync, scale_factor);

    ConfigFile {
        window: window_config,
        audio: AudioConfig::default(), // TODO: Read from Bevy audio when implemented
        game: game_config.clone(),
    }
}

/// Builds WindowConfig while preserving resolutions for other window modes.
///
/// This loads the existing config from localStorage to preserve resolution
/// settings for window modes we're not currently in. For example, if we're
/// in windowed mode, we want to preserve the fullscreen and borderless resolutions.
///
/// # Arguments
///
/// * `current_mode` - The current window mode
/// * `current_resolution` - The current window resolution
/// * `vsync` - The current VSync mode
/// * `scale_factor` - The current scale factor override
///
/// # Returns
///
/// A WindowConfig with current mode's resolution updated, others preserved
fn build_window_config_preserving_other_modes(
    current_mode: WindowMode,
    current_resolution: Resolution,
    vsync: VsyncMode,
    scale_factor: Option<f64>,
) -> WindowConfig {
    // Load existing config to preserve other modes' resolutions
    let existing = match storage::load_config() {
        Ok(contents) => toml::from_str::<ConfigFile>(&contents)
            .map(|c| c.window)
            .unwrap_or_default(),
        Err(_) => WindowConfig::default(),
    };

    // Update only the current mode's resolution
    let (windowed_res, borderless_res, fullscreen_res) = match current_mode {
        WindowMode::Windowed => (
            current_resolution,
            existing.borderless_resolution,
            existing.fullscreen_resolution,
        ),
        WindowMode::Borderless => (
            existing.windowed_resolution,
            current_resolution,
            existing.fullscreen_resolution,
        ),
        WindowMode::Fullscreen => (
            existing.windowed_resolution,
            existing.borderless_resolution,
            current_resolution,
        ),
    };

    WindowConfig {
        windowed_resolution: windowed_res,
        borderless_resolution: borderless_res,
        fullscreen_resolution: fullscreen_res,
        mode: current_mode,
        vsync,
        scale_factor,
    }
}
