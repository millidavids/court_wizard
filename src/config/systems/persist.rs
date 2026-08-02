use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window as BevyWindow};

use super::super::messages::*;
use super::super::resources::*;
use super::super::save_data;
use super::super::storage;
use super::window::get_window_position;

/// Detects GameConfig changes and triggers config save.
///
/// This system monitors the GameConfig resource for changes and emits
/// a ConfigChanged message to trigger the debounce timer for saving.
///
/// # Arguments
///
/// * `game_config` - Game configuration resource
/// * `config_changed` - Message writer for config changed messages
pub(crate) fn detect_game_config_changes(
    game_config: Res<GameConfig>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if !game_config.is_changed() {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Detects InputBindings changes and triggers config save.
pub(crate) fn detect_input_bindings_changes(
    bindings: Res<super::super::input_bindings::InputBindings>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if !bindings.is_changed() {
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
pub(crate) fn mark_save_on_config_changed(
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn save_config_on_debounce_timer(
    time: Res<Time>,
    mut debounce_timer: ResMut<SaveDebounceTimer>,
    game_config: Res<GameConfig>,
    active_save: Res<ActiveSave>,
    saved_geometry: Res<SavedWindowedGeometry>,
    input_bindings: Res<super::super::input_bindings::InputBindings>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    time_travel: Option<Res<crate::game::time_travel::TimeTravelState>>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if !debounce_timer.pending {
        return;
    }

    debounce_timer.timer.tick(time.delta());

    if debounce_timer.timer.is_finished() {
        persist_current_state(
            &game_config,
            &active_save,
            &saved_geometry,
            &input_bindings,
            &game_mode,
            time_travel.is_some(),
            &windows,
        );
        debounce_timer.pending = false;
    }
}

/// Manual save trigger (bypasses debounce).
///
/// Listens for SaveConfigMessage and immediately saves + flushes to disk.
#[allow(clippy::too_many_arguments)]
pub(crate) fn save_config_on_event(
    mut save_events: MessageReader<SaveConfigMessage>,
    game_config: Res<GameConfig>,
    active_save: Res<ActiveSave>,
    saved_geometry: Res<SavedWindowedGeometry>,
    input_bindings: Res<super::super::input_bindings::InputBindings>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    time_travel: Option<Res<crate::game::time_travel::TimeTravelState>>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if save_events.read().count() == 0 {
        return;
    }

    persist_current_state(
        &game_config,
        &active_save,
        &saved_geometry,
        &input_bindings,
        &game_mode,
        time_travel.is_some(),
        &windows,
    );
    save_data::flush_save_cache();
}

/// Periodically flushes the in-memory save cache to disk.
/// Runs every 2 seconds when the cache has unflushed changes.
pub(crate) fn periodic_save_flush(time: Res<Time>, mut timer: Local<Option<Timer>>) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(2.0, TimerMode::Repeating));
    timer.tick(time.delta());
    if timer.just_finished() {
        save_data::flush_save_cache();
    }
}

/// Flushes all pending saves to disk when the app is exiting.
///
/// Catches Alt+F4, window close, and explicit exit to prevent data loss.
/// Runs in `Last` so it executes after the exit message is written.
#[allow(clippy::too_many_arguments)]
pub(crate) fn save_on_exit(
    mut exit_events: MessageReader<AppExit>,
    game_config: Res<GameConfig>,
    active_save: Res<ActiveSave>,
    saved_geometry: Res<SavedWindowedGeometry>,
    input_bindings: Res<super::super::input_bindings::InputBindings>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    time_travel: Option<Res<crate::game::time_travel::TimeTravelState>>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if exit_events.read().count() == 0 {
        return;
    }

    info!("App exiting — flushing saves to disk");
    persist_current_state(
        &game_config,
        &active_save,
        &saved_geometry,
        &input_bindings,
        &game_mode,
        time_travel.is_some(),
        &windows,
    );
    save_data::flush_save_cache();
}

/// Forces the OS process to terminate immediately after `save_on_exit` has
/// flushed state to disk. Without this, Bevy's normal `app.run()` shutdown
/// path can stall for tens of seconds on macOS after a played session —
/// audio/WGPU/network resources accumulated during gameplay are slow to drop,
/// and the in-process drop chain never reaches `main()`. Since saves are
/// already persisted and the process is exiting, skipping the remaining
/// Drop chain is safe.
pub(crate) fn force_exit_after_save(mut exit_events: MessageReader<AppExit>) {
    if exit_events.read().count() > 0 {
        std::process::exit(0);
    }
}

/// Resolves current game state and persists config + save data to disk.
fn persist_current_state(
    game_config: &GameConfig,
    active_save: &ActiveSave,
    saved_geometry: &SavedWindowedGeometry,
    input_bindings: &super::super::input_bindings::InputBindings,
    game_mode: &Option<Res<crate::game::game_mode::components::GameMode>>,
    is_time_travel: bool,
    windows: &Query<&BevyWindow, With<PrimaryWindow>>,
) {
    // Roguelite runs and time-travel replays must both leave the wizard's Endless
    // progression untouched — see `save_config_to_active_wizard`.
    let skip_progression =
        crate::game::game_mode::components::is_roguelite_mode(game_mode.as_deref())
            || is_time_travel;
    persist_config(
        game_config,
        active_save,
        saved_geometry,
        input_bindings,
        get_window_position(windows, saved_geometry),
        skip_progression,
    );
}

/// Builds a ConfigFile from current state, serializes to TOML, and saves to disk.
fn persist_config(
    game_config: &GameConfig,
    active_save: &ActiveSave,
    saved_geometry: &SavedWindowedGeometry,
    input_bindings: &super::super::input_bindings::InputBindings,
    window_pos: Option<IVec2>,
    skip_progression: bool,
) {
    // Build ConfigFile from current state
    let config_file =
        build_config_from_game_config(game_config, saved_geometry, input_bindings, window_pos);

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
    save_data::save_config_to_active_wizard(game_config, active_save, skip_progression);
}

/// Builds a temporary ConfigFile from current GameConfig for serialization.
fn build_config_from_game_config(
    game_config: &GameConfig,
    saved_geometry: &SavedWindowedGeometry,
    input_bindings: &super::super::input_bindings::InputBindings,
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
        controls: input_bindings.clone(),
    }
}
