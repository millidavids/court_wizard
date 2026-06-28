use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window as BevyWindow, WindowPosition};

use super::super::resources::*;
use super::super::save_data;
use super::super::storage;
use super::window::{apply_display_mode_to_window, apply_vsync_config};

/// Loads configuration from disk at startup and applies settings.
/// Falls back to sensible defaults if the config file is missing or invalid.
pub(crate) fn load_and_apply_config(
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
        colorblind_type: config_file.game.colorblind_type,
        colorblind_strength: config_file.game.colorblind_strength,
        reduce_flashes: config_file.game.reduce_flashes,
        reduce_motion: config_file.game.reduce_motion,
        crt_enabled: config_file.game.crt_enabled,
        game_speed: config_file.game.game_speed.clamp(0.5, 2.0),
        auto_pause_on_focus_loss: config_file.game.auto_pause_on_focus_loss,
        pause_on_steam_overlay: config_file.game.pause_on_steam_overlay,
        pause_on_controller_disconnect: config_file.game.pause_on_controller_disconnect,
        high_contrast_strength: config_file.game.high_contrast_strength,
        aim_assist: config_file.game.aim_assist,
        gamepad_sensitivity_x: config_file.game.gamepad_sensitivity_x,
        gamepad_sensitivity_y: config_file.game.gamepad_sensitivity_y,
        gamepad_deadzone: config_file.game.gamepad_deadzone,
        gamepad_response_curve: config_file.game.gamepad_response_curve,
        rumble_enabled: config_file.game.rumble_enabled,
        controller_glyph_style: config_file.game.controller_glyph_style,
        saved_walls: Vec::new(),
        saved_crystals: Vec::new(),
        saved_flora: Vec::new(),
        saved_trampling: Default::default(),
        saved_trees: Vec::new(),
        saved_ponds: Vec::new(),
        saved_bushes: Vec::new(),
        saved_boulders: Vec::new(),
        seed: None,
    };
    // Migrate legacy saves into unified save file if needed
    save_data::migrate_legacy_saves(&game_config);

    // Progress fields in GameConfig are only meaningful after loading a wizard.
    // Reset them to defaults here; they get populated when a wizard is loaded.
    game_config.current_level = 1;
    game_config.highest_level_achieved = 1;
    game_config.efficiency_ratios = std::collections::HashMap::new();

    commands.insert_resource(game_config);
    commands.insert_resource(config_file.controls);

    // ConfigFile is now discarded - GameConfig is the source of truth
}
