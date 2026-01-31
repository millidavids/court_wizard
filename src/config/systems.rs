use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, Window as BevyWindow, WindowResized};

use super::progress;
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

    // Apply VSync to Bevy's Window
    let Ok(mut window) = windows.single_mut() else {
        warn!("Could not find primary window to apply config");
        return;
    };
    apply_vsync_config(config_file.window.vsync, &mut window);

    // Create GameConfig resource from config file
    let mut game_config = GameConfig {
        vsync: config_file.window.vsync,
        master_volume: config_file.audio.master_volume,
        music_volume: config_file.audio.music_volume,
        sfx_volume: config_file.audio.sfx_volume,
        difficulty: config_file.game.difficulty,
        brightness: config_file.game.brightness.max(0.1), // Ensure minimum 10% to prevent soft-lock
        current_level: config_file.game.current_level,
        highest_level_achieved: config_file.game.highest_level_achieved,
        efficiency_ratios: config_file.game.efficiency_ratios,
    };
    // Verify progress against signed copy in localStorage
    match progress::load_verified_progress() {
        Some(verified) => {
            game_config.current_level = verified.current_level;
            game_config.highest_level_achieved = verified.highest_level_achieved;
            game_config.efficiency_ratios = verified.efficiency_ratios;
            info!("Loaded verified progress from signed storage");
        }
        None => {
            warn!("No valid signed progress found, resetting progress to defaults");
            game_config.current_level = 1;
            game_config.highest_level_achieved = 1;
            game_config.efficiency_ratios = std::collections::HashMap::new();
        }
    }

    commands.insert_resource(game_config);

    // ConfigFile is now discarded - GameConfig is the source of truth
}

/// Applies VSync configuration to Bevy's Window component.
///
/// # Arguments
///
/// * `vsync` - VSync mode from config
/// * `window` - Mutable reference to Bevy's Window component
fn apply_vsync_config(vsync: VsyncMode, window: &mut BevyWindow) {
    window.present_mode = match vsync {
        VsyncMode::Off => PresentMode::AutoNoVsync,
        VsyncMode::Adaptive => PresentMode::AutoVsync,
        VsyncMode::On => PresentMode::AutoVsync,
    };

    info!("Applied VSync config: {:?}", vsync);
}

/// Detects window resize events and triggers config save.
///
/// This system monitors Bevy's WindowResized events and emits a ConfigChanged
/// message to trigger the debounce timer for saving.
///
/// # Arguments
///
/// * `resize_events` - Message reader for window resize events
/// * `config_changed` - Message writer for config changed messages
pub fn detect_window_resize(
    mut resize_events: MessageReader<WindowResized>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if resize_events.read().count() == 0 {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Detects GameConfig changes and triggers config save.
///
/// This system monitors the GameConfig resource for changes and emits
/// a ConfigChanged message to trigger the debounce timer for saving.
///
/// # Arguments
///
/// * `game_config` - Game configuration resource
/// * `config_changed` - Message writer for config changed messages
pub fn detect_game_config_changes(
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
    game_config: Res<GameConfig>,
) {
    if !debounce_timer.pending {
        return;
    }

    debounce_timer.timer.tick(time.delta());

    if debounce_timer.timer.is_finished() {
        persist_config(&game_config);
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
    game_config: Res<GameConfig>,
) {
    if save_events.read().count() == 0 {
        return;
    }

    persist_config(&game_config);
}

/// Saves current state to localStorage by reading from Bevy components.
///
/// This function reads the current state from:
/// - Bevy's Window component (window settings)
/// - WindowConfig resource (window mode and vsync settings)
/// - AudioConfig resource (volume settings)
/// - GameConfig resource (game settings)
///
/// Then builds a temporary ConfigFile, serializes to TOML, and saves to localStorage.
///
/// # Arguments
///
/// * `windows` - Query for the primary window
/// * `window_config` - Window configuration resource
/// * `audio_config` - Audio configuration resource
/// * `game_config` - Game configuration resource
fn persist_config(game_config: &GameConfig) {
    // Build ConfigFile from current state
    let config_file = build_config_from_game_config(game_config);

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

    // Also save signed progress
    progress::save_signed_progress(game_config);
}

/// Builds ConfigFile from current GameConfig.
///
/// This function constructs a temporary ConfigFile for serialization.
/// The ConfigFile is immediately discarded after serialization - it's not kept in memory.
///
/// # Arguments
///
/// * `game_config` - Reference to the GameConfig resource
///
/// # Returns
///
/// A ConfigFile struct populated with current settings
fn build_config_from_game_config(game_config: &GameConfig) -> ConfigFile {
    // Load existing config to preserve window settings we don't modify (resolution, etc.)
    let existing_window = match storage::load_config() {
        Ok(contents) => toml::from_str::<ConfigFile>(&contents)
            .map(|c| c.window)
            .unwrap_or_default(),
        Err(_) => WindowConfig::default(),
    };

    // Update only the VSync setting, preserve everything else
    let window_config = WindowConfig {
        vsync: game_config.vsync,
        ..existing_window
    };

    let audio_config = AudioConfig {
        master_volume: game_config.master_volume,
        music_volume: game_config.music_volume,
        sfx_volume: game_config.sfx_volume,
    };

    ConfigFile {
        window: window_config,
        audio: audio_config,
        game: game_config.clone(),
    }
}
