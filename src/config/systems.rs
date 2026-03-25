use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, Window as BevyWindow, WindowMode, WindowMoved, WindowPosition, WindowResized};

use super::messages::*;
use super::resources::*;
use super::save_data;
use super::storage;

/// Loads configuration from disk at startup and applies settings.
/// Falls back to sensible defaults if the config file is missing or invalid.
pub(super) fn load_and_apply_config(
    mut commands: Commands,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
) {
    let config_file = match storage::load_config() {
        Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
            Ok(config) => {
                info!("Loaded config from disk");
                config
            }
            Err(e) => {
                warn!("Failed to parse config: {}, using defaults", e);
                ConfigFile::default()
            }
        },
        Err(_) => {
            info!("No config file found, using defaults");
            let config = ConfigFile::default();
            // Save defaults to disk
            if let Ok(toml_string) = toml::to_string_pretty(&config) {
                let _ = storage::save_config(&toml_string);
            }
            config
        }
    };

    // Apply window settings
    let Ok(mut window) = windows.single_mut() else {
        warn!("Could not find primary window to apply config");
        return;
    };
    apply_vsync_config(config_file.window.vsync, &mut window);
    apply_display_mode_to_window(config_file.window.display_mode, &mut window);

    // Initialize saved windowed geometry from config
    let saved_pos = match (config_file.window.position_x, config_file.window.position_y) {
        (Some(x), Some(y)) => Some(IVec2::new(x, y)),
        _ => None,
    };
    commands.insert_resource(SavedWindowedGeometry {
        width: config_file.window.windowed_width,
        height: config_file.window.windowed_height,
        position: saved_pos,
        pending_mode_change: None,
    });

    // Restore saved window position
    if let (Some(x), Some(y)) = (config_file.window.position_x, config_file.window.position_y) {
        window.position = WindowPosition::At(IVec2::new(x, y));
    }

    // Create GameConfig resource from config file
    let mut game_config = GameConfig {
        vsync: config_file.window.vsync,
        display_mode: config_file.window.display_mode,
        master_volume: config_file.audio.master_volume,
        music_volume: config_file.audio.music_volume,
        sfx_volume: config_file.audio.sfx_volume,
        difficulty: config_file.game.difficulty,
        brightness: config_file.game.brightness.max(0.1), // Ensure minimum 10% to prevent soft-lock
        current_level: config_file.game.current_level,
        highest_level_achieved: config_file.game.highest_level_achieved,
        efficiency_ratios: config_file.game.efficiency_ratios,
        action_bar_slots: config_file.game.action_bar_slots,
        wizard_type: config_file.game.wizard_type,
        skip_splash: config_file.game.skip_splash,
        tutorials_enabled: config_file.game.tutorials_enabled,
        show_level_clock: config_file.game.show_level_clock,
        urgent_mode: config_file.game.urgent_mode,
        saved_walls: Vec::new(),
        saved_crystals: Vec::new(),
    };
    // Migrate legacy saves into unified save file if needed
    save_data::migrate_legacy_saves(&game_config);

    // Progress fields in GameConfig are only meaningful after loading a wizard.
    // Reset them to defaults here; they get populated when a wizard is loaded.
    game_config.current_level = 1;
    game_config.highest_level_achieved = 1;
    game_config.efficiency_ratios = std::collections::HashMap::new();

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

/// Applies display mode configuration to Bevy's Window component.
fn apply_display_mode_to_window(display_mode: DisplayMode, window: &mut BevyWindow) {
    window.mode = match display_mode {
        DisplayMode::Windowed => WindowMode::Windowed,
        DisplayMode::BorderlessFullscreen => {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        }
    };
    info!("Applied display mode: {:?}", display_mode);
}

/// Reactively applies vsync and display mode when the relevant fields change.
///
/// Tracks the previous values to avoid redundant Window mutations (which would
/// cause unnecessary logging and potential flicker) when unrelated GameConfig
/// fields change (e.g. volume sliders).
pub(super) fn apply_display_mode(
    game_config: Res<GameConfig>,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    mut last_vsync: Local<Option<VsyncMode>>,
    mut last_display_mode: Local<Option<DisplayMode>>,
) {
    let vsync_changed = *last_vsync != Some(game_config.vsync);
    let display_changed = *last_display_mode != Some(game_config.display_mode);

    if !vsync_changed && !display_changed {
        return;
    }

    // VSync can be applied immediately (no render target size change)
    if vsync_changed && !display_changed {
        if let Ok(mut window) = windows.single_mut() {
            apply_vsync_config(game_config.vsync, &mut window);
        }
        *last_vsync = Some(game_config.vsync);
    }

    if display_changed {
        let was_windowed = matches!(
            last_display_mode.as_ref(),
            Some(DisplayMode::Windowed) | None
        );

        // Save current windowed geometry before switching away from windowed mode
        if was_windowed && game_config.display_mode != DisplayMode::Windowed {
            if let Ok(window) = windows.single() {
                saved_geometry.width = window.resolution.width();
                saved_geometry.height = window.resolution.height();
                saved_geometry.position = match window.position {
                    WindowPosition::At(pos) => Some(pos),
                    _ => None,
                };
                info!(
                    "Saved windowed geometry: {}x{} at {:?}",
                    saved_geometry.width, saved_geometry.height, saved_geometry.position
                );
            }
        }

        // Defer the actual mode change to the next frame to avoid wgpu
        // scissor rect / render target size mismatch crashes.
        saved_geometry.pending_mode_change = Some(game_config.display_mode);
        *last_vsync = Some(game_config.vsync);
        *last_display_mode = Some(game_config.display_mode);
    }
}

/// Applies deferred display mode changes one frame after they were requested.
///
/// This avoids wgpu validation errors where the scissor rect from the previous
/// frame's resolution doesn't fit the new render target size.
pub(super) fn apply_deferred_mode_change(
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    game_config: Res<GameConfig>,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Camera3d>>,
) {
    let Some(new_mode) = saved_geometry.pending_mode_change.take() else {
        return;
    };

    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    // Clear the camera viewport before ANY mode change. The OS resizes
    // the window surface asynchronously, so enforce_aspect_ratio may see
    // stale dimensions and set a viewport that doesn't match the actual
    // render target, causing a wgpu scissor rect crash. Setting viewport
    // to None lets the camera use the full surface for one frame until
    // enforce_aspect_ratio recomputes with correct dimensions.
    for mut camera in &mut cameras {
        camera.viewport = None;
    }

    apply_display_mode_to_window(new_mode, &mut window);

    if new_mode == DisplayMode::Windowed {
        window
            .resolution
            .set(saved_geometry.width, saved_geometry.height);
        if let Some(pos) = saved_geometry.position {
            window.position = WindowPosition::At(pos);
        }
        info!(
            "Restored windowed geometry: {}x{} at {:?}",
            saved_geometry.width, saved_geometry.height, saved_geometry.position
        );
    }

    // Always apply current vsync when changing mode
    apply_vsync_config(game_config.vsync, &mut window);
}

/// Detects window resize events and triggers config save.
pub(super) fn detect_window_resize(
    mut resize_events: MessageReader<WindowResized>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if resize_events.read().count() == 0 {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Detects window move events and saves the new position to config.
pub(super) fn detect_window_move(
    mut move_events: MessageReader<WindowMoved>,
    mut config_changed: MessageWriter<ConfigChanged>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if move_events.read().count() == 0 {
        return;
    }

    // Only save if the window is actually visible on screen (ignore minimized/offscreen moves)
    let Ok(window) = windows.single() else {
        return;
    };
    if window.physical_width() == 0 || window.physical_height() == 0 {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Extracts the saved window position from the primary window, if set.
fn get_window_position(windows: &Query<&BevyWindow, With<PrimaryWindow>>) -> Option<IVec2> {
    windows.single().ok().and_then(|w| match w.position {
        WindowPosition::At(pos) => Some(pos),
        _ => None,
    })
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
pub(super) fn detect_game_config_changes(
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
pub(super) fn mark_save_on_config_changed(
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

/// Ticks debounce timer and saves to disk when expired.
///
/// This system runs every frame during the `Update` schedule. When the
/// debounce timer expires (0.5s of no config changes), it reads the current
/// state from Bevy components and saves to disk.
///
/// # Arguments
///
/// * `time` - Time resource for delta time
/// * `debounce_timer` - Debounce timer resource
/// * `windows` - Query for the primary window
/// * `game_config` - Game configuration resource
pub(super) fn save_config_on_debounce_timer(
    time: Res<Time>,
    mut debounce_timer: ResMut<SaveDebounceTimer>,
    game_config: Res<GameConfig>,
    active_save: Res<ActiveSave>,
    saved_geometry: Res<SavedWindowedGeometry>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if !debounce_timer.pending {
        return;
    }

    debounce_timer.timer.tick(time.delta());

    if debounce_timer.timer.is_finished() {
        let is_roguelite =
            crate::game::game_mode::components::is_roguelite_mode(game_mode.as_deref());
        persist_config(
            &game_config,
            &active_save,
            &saved_geometry,
            get_window_position(&windows),
            is_roguelite,
        );
        debounce_timer.pending = false;
    }
}

/// Manual save trigger (bypasses debounce).
///
/// This system listens for SaveConfigMessage messages and immediately
/// saves the current state to disk, bypassing the debounce timer.
/// Useful for critical saves (e.g., on app quit).
///
/// # Arguments
///
/// * `save_events` - Message reader for save config events
/// * `windows` - Query for the primary window
/// * `game_config` - Game configuration resource
pub(super) fn save_config_on_event(
    mut save_events: MessageReader<SaveConfigMessage>,
    game_config: Res<GameConfig>,
    active_save: Res<ActiveSave>,
    saved_geometry: Res<SavedWindowedGeometry>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if save_events.read().count() == 0 {
        return;
    }

    let is_roguelite =
        crate::game::game_mode::components::is_roguelite_mode(game_mode.as_deref());
    persist_config(
        &game_config,
        &active_save,
        &saved_geometry,
        get_window_position(&windows),
        is_roguelite,
    );

    // Force-flush the save cache to disk immediately on manual save
    save_data::flush_save_cache();
}

/// Periodically flushes the in-memory save cache to disk.
/// Runs every 2 seconds when the cache has unflushed changes.
pub(super) fn periodic_save_flush(
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(2.0, TimerMode::Repeating));
    timer.tick(time.delta());
    if timer.just_finished() {
        save_data::flush_save_cache();
    }
}

/// Builds a ConfigFile from current state, serializes to TOML, and saves to disk.
fn persist_config(
    game_config: &GameConfig,
    active_save: &ActiveSave,
    saved_geometry: &SavedWindowedGeometry,
    window_pos: Option<IVec2>,
    is_roguelite: bool,
) {
    // Build ConfigFile from current state
    let config_file = build_config_from_game_config(game_config, saved_geometry, window_pos);

    // Serialize and save
    match toml::to_string_pretty(&config_file) {
        Ok(toml_string) => match storage::save_config(&toml_string) {
            Ok(_) => {
                info!("Config saved to disk");
            }
            Err(e) => {
                error!("Failed to save config: {}", e);
            }
        },
        Err(e) => {
            error!("Failed to serialize config: {}", e);
        }
    }

    // Save progress to the active wizard in the unified save file
    save_data::save_config_to_active_wizard(game_config, active_save, is_roguelite);
}

/// Builds a temporary ConfigFile from current GameConfig for serialization.
fn build_config_from_game_config(
    game_config: &GameConfig,
    saved_geometry: &SavedWindowedGeometry,
    window_pos: Option<IVec2>,
) -> ConfigFile {
    let window_config = WindowConfig {
        vsync: game_config.vsync,
        display_mode: game_config.display_mode,
        position_x: window_pos.map(|p| p.x),
        position_y: window_pos.map(|p| p.y),
        windowed_width: saved_geometry.width,
        windowed_height: saved_geometry.height,
        ..WindowConfig::default()
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
