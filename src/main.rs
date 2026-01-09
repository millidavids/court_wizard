use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
use std::fs;

mod config;
mod state;

use config::{ConfigFile, ConfigPlugin, VsyncMode};
use state::GameState;

/// Main entry point for the game.
///
/// Pre-loads the configuration file to set initial window properties,
/// then initializes the Bevy app with the config plugin.
fn main() {
    // Pre-load config for initial window setup
    let config = load_initial_config();

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "The Game".into(),
                    resolution: WindowResolution::new(config.window.width, config.window.height)
                        .with_scale_factor_override(
                            config.window.scale_factor.unwrap_or(1.0) as f32
                        ),
                    present_mode: match config.window.vsync {
                        VsyncMode::Off => PresentMode::AutoNoVsync,
                        VsyncMode::Adaptive => PresentMode::AutoVsync,
                        VsyncMode::On => PresentMode::AutoVsync,
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(ConfigPlugin::default())
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .run();
}

/// Loads the configuration file before App initialization.
///
/// This function is called before Bevy's app is created to allow the
/// initial window properties to be set from the config file. If the
/// config file doesn't exist or cannot be parsed, defaults are used.
///
/// # Returns
///
/// The loaded `ConfigFile` or defaults if loading fails.
fn load_initial_config() -> ConfigFile {
    let config_path = "config.toml";
    if std::path::Path::new(config_path).exists()
        && let Ok(contents) = fs::read_to_string(config_path)
        && let Ok(config) = toml::from_str::<ConfigFile>(&contents)
    {
        return config;
    }
    ConfigFile::default()
}

/// Sets up the initial game scene.
///
/// Spawns the primary 2D camera required for rendering.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for spawning entities
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
